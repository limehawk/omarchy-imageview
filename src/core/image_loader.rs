use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use image::codecs::gif::GifDecoder;
use image::codecs::webp::WebPDecoder;
use image::{AnimationDecoder, DynamicImage, Frame, RgbaImage};
use super::formats::{detect_format, FormatGroup};

/// Hard ceiling for one SVG raster. Viewport draws use this too.
pub const MAX_SVG_EDGE: u32 = 8192;

/// Load an image file, returning None on failure or unsupported format.
pub fn load_image(path: &Path) -> Option<DynamicImage> {
    if !path.is_file() {
        log::warn!("load_image: not a file: {}", path.display());
        return None;
    }

    let filename = path.file_name()?.to_string_lossy();
    let fmt = detect_format(&filename)?;

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let img = match fmt {
        FormatGroup::Svg => load_svg(path)?,
        FormatGroup::Image => match ext.as_str() {
            // libheif already applies irot/imir; don't also honor EXIF.
            "heic" | "heif" => load_heif(path)?,
            "jxl" => apply_exif_orientation(load_jxl(path)?, path),
            _ => match image::open(path) {
                Ok(img) => apply_exif_orientation(img, path),
                Err(e) => {
                    log::error!("load_image: decode failed for {}: {e}", path.display());
                    return None;
                }
            },
        },
    };
    log::debug!(
        "load_image: decoded {} ({}x{})",
        path.display(),
        img.width(),
        img.height()
    );
    Some(img)
}

pub fn svg_exceeds_cap(w: u32, h: u32) -> bool {
    w > MAX_SVG_EDGE || h > MAX_SVG_EDGE
}

/// Pixel size from the opening `<svg>` tag. Width/height win over viewBox.
pub fn svg_intrinsic_size(svg: &str) -> Option<(u32, u32)> {
    let start = svg.find("<svg")?;
    let rest = &svg[start..];
    let end = rest.find('>')?;
    let tag = &rest[..end];
    if let (Some(w), Some(h)) = (attr_len(tag, "width"), attr_len(tag, "height")) {
        return Some((w, h));
    }
    view_box_size(tag)
}

pub fn svg_target_size(
    view_w: u32,
    view_h: u32,
    zoom_fit: bool,
    zoom: f64,
    intrinsic: Option<(u32, u32)>,
) -> (u32, u32) {
    let cap = |v: u32| v.clamp(1, MAX_SVG_EDGE);
    if zoom_fit {
        return (cap(view_w), cap(view_h));
    }
    if (zoom - 1.0).abs() < f64::EPSILON {
        if let Some((iw, ih)) = intrinsic {
            return (cap(iw), cap(ih));
        }
    }
    let z = zoom.max(0.1);
    (
        cap(((view_w as f64) * z).ceil() as u32),
        cap(((view_h as f64) * z).ceil() as u32),
    )
}

fn load_svg(path: &Path) -> Option<DynamicImage> {
    let head = std::fs::read_to_string(path).ok()?;
    if let Some((w, h)) = svg_intrinsic_size(&head) {
        if svg_exceeds_cap(w, h) {
            log::warn!(
                "load_svg: cap {} ({}x{}, max {})",
                path.display(),
                w,
                h,
                MAX_SVG_EDGE
            );
            return load_svg_at(path, MAX_SVG_EDGE, MAX_SVG_EDGE);
        }
    }
    raster_svg(path, None)
}

pub fn load_svg_at(path: &Path, max_w: u32, max_h: u32) -> Option<DynamicImage> {
    let max_w = max_w.clamp(1, MAX_SVG_EDGE);
    let max_h = max_h.clamp(1, MAX_SVG_EDGE);
    raster_svg(path, Some((max_w, max_h)))
}

fn raster_svg(path: &Path, max: Option<(u32, u32)>) -> Option<DynamicImage> {
    let mut cmd = std::process::Command::new("rsvg-convert");
    if let Some((w, h)) = max {
        cmd.args(["-a", "-w", &w.to_string(), "-h", &h.to_string()]);
    }
    let out = match cmd.arg(path).output() {
        Ok(o) => o,
        Err(e) => {
            log::error!("load_svg: rsvg-convert failed for {}: {e}", path.display());
            return None;
        }
    };
    if !out.status.success() {
        log::warn!(
            "load_svg: rsvg-convert exit {} for {}",
            out.status,
            path.display()
        );
        return None;
    }
    match image::load_from_memory(&out.stdout) {
        Ok(img) => Some(img),
        Err(e) => {
            log::error!("load_svg: png decode failed for {}: {e}", path.display());
            None
        }
    }
}

fn attr_len(tag: &str, name: &str) -> Option<u32> {
    parse_len(attr_raw(tag, name)?)
}

fn attr_raw<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let dq = format!("{name}=\"");
    if let Some(i) = tag.find(&dq) {
        let rest = &tag[i + dq.len()..];
        return rest.split('"').next();
    }
    let sq = format!("{name}='");
    if let Some(i) = tag.find(&sq) {
        let rest = &tag[i + sq.len()..];
        return rest.split('\'').next();
    }
    None
}

fn parse_len(s: &str) -> Option<u32> {
    let s = s.trim();
    if s.ends_with('%') {
        return None;
    }
    let s = s.trim_end_matches("px");
    let f: f64 = s.parse().ok()?;
    if !f.is_finite() || f <= 0.0 || f > u32::MAX as f64 {
        return None;
    }
    Some(f.ceil() as u32)
}

fn view_box_size(tag: &str) -> Option<(u32, u32)> {
    let raw = attr_raw(tag, "viewBox").or_else(|| attr_raw(tag, "viewbox"))?;
    let nums: Vec<f64> = raw
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();
    if nums.len() != 4 {
        return None;
    }
    let (w, h) = (nums[2], nums[3]);
    if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
        return None;
    }
    if w > u32::MAX as f64 || h > u32::MAX as f64 {
        return None;
    }
    Some((w.ceil() as u32, h.ceil() as u32))
}

/// Apply a TIFF/EXIF orientation tag (1–8) to a decoded image.
pub fn apply_orientation(img: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    }
}

/// Compose an existing EXIF orientation with a clockwise display rotation.
pub fn compose_orientation_cw(orientation: u32, degrees: u32) -> u32 {
    let mut tag = if (1..=8).contains(&orientation) { orientation } else { 1 };
    let steps = (degrees / 90) % 4;
    for _ in 0..steps {
        tag = match tag {
            1 => 6,
            2 => 7,
            3 => 8,
            4 => 5,
            5 => 2,
            6 => 3,
            7 => 4,
            8 => 1,
            _ => 6,
        };
    }
    tag
}

/// Mirror across the vertical axis (left-right).
pub fn compose_flip_h(orientation: u32) -> u32 {
    match if (1..=8).contains(&orientation) { orientation } else { 1 } {
        1 => 2,
        2 => 1,
        3 => 4,
        4 => 3,
        5 => 6,
        6 => 5,
        7 => 8,
        8 => 7,
        _ => 2,
    }
}

/// Mirror across the horizontal axis (up-down).
pub fn compose_flip_v(orientation: u32) -> u32 {
    match if (1..=8).contains(&orientation) { orientation } else { 1 } {
        1 => 4,
        2 => 3,
        3 => 2,
        4 => 1,
        5 => 8,
        6 => 7,
        7 => 6,
        8 => 5,
        _ => 4,
    }
}

/// Rotation then optional flips, composed onto an existing EXIF tag.
pub fn compose_transform(orientation: u32, degrees: u32, flip_h: bool, flip_v: bool) -> u32 {
    let mut tag = compose_orientation_cw(orientation, degrees);
    if flip_h {
        tag = compose_flip_h(tag);
    }
    if flip_v {
        tag = compose_flip_v(tag);
    }
    tag
}

pub fn rotate_degrees(img: DynamicImage, degrees: u32) -> DynamicImage {
    match degrees % 360 {
        90 => img.rotate90(),
        180 => img.rotate180(),
        270 => img.rotate270(),
        _ => img,
    }
}

pub fn read_exif_orientation(path: &Path) -> Option<u32> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let exif = exif::Reader::new().read_from_container(&mut reader).ok()?;
    let field = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?;
    field.value.get_uint(0)
}

fn apply_exif_orientation(img: DynamicImage, path: &Path) -> DynamicImage {
    match read_exif_orientation(path) {
        Some(tag) if (2..=8).contains(&tag) => apply_orientation(img, tag),
        _ => img,
    }
}

fn load_heif(path: &Path) -> Option<DynamicImage> {
    let path_str = match path.to_str() {
        Some(s) => s,
        None => {
            log::error!("load_heif: non-utf8 path {}", path.display());
            return None;
        }
    };

    let lib = libheif_rs::LibHeif::new();
    let ctx = match libheif_rs::HeifContext::read_from_file(path_str) {
        Ok(c) => c,
        Err(e) => {
            log::error!("load_heif: open failed for {path_str}: {e}");
            return None;
        }
    };
    let handle = match ctx.primary_image_handle() {
        Ok(h) => h,
        Err(e) => {
            log::error!("load_heif: no primary image in {path_str}: {e}");
            return None;
        }
    };
    let decoded = match lib.decode(
        &handle,
        libheif_rs::ColorSpace::Rgb(libheif_rs::RgbChroma::Rgba),
        None,
    ) {
        Ok(img) => img,
        Err(e) => {
            log::error!("load_heif: decode failed for {path_str}: {e}");
            return None;
        }
    };
    let plane = match decoded.planes().interleaved {
        Some(p) => p,
        None => {
            log::error!("load_heif: no interleaved plane in {path_str}");
            return None;
        }
    };

    let w = plane.width;
    let h = plane.height;
    let stride = plane.stride;
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h as usize {
        let start = y * stride;
        rgba.extend_from_slice(&plane.data[start..start + w as usize * 4]);
    }
    RgbaImage::from_raw(w, h, rgba).map(DynamicImage::ImageRgba8)
}

fn load_jxl(path: &Path) -> Option<DynamicImage> {
    let image = match jxl_oxide::JxlImage::builder().open(path) {
        Ok(img) => img,
        Err(e) => {
            log::error!("load_jxl: open failed for {}: {e}", path.display());
            return None;
        }
    };
    let render = match image.render_frame(0) {
        Ok(r) => r,
        Err(e) => {
            log::error!("load_jxl: render failed for {}: {e}", path.display());
            return None;
        }
    };
    let mut stream = render.stream();
    let w = stream.width();
    let h = stream.height();
    let mut rgba = vec![0u8; w as usize * h as usize * 4];
    stream.write_to_buffer(&mut rgba);
    RgbaImage::from_raw(w, h, rgba).map(DynamicImage::ImageRgba8)
}

/// One decoded animation frame: raw RGBA bytes, dimensions, and per-frame delay in ms.
pub struct AnimatedFrame {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub delay_ms: u32,
}

/// Try to load `path` as an animation. Returns `Some(frames)` only if the file
/// is genuinely animated (multi-frame gif, or webp with `has_animation()`).
/// Returns `None` for static images so the caller can fall through to its
/// regular static-decode path.
pub fn load_animated_frames(path: &Path) -> Option<Vec<AnimatedFrame>> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())?
        .to_ascii_lowercase();

    let frames = match ext.as_str() {
        "gif" => decode_gif_frames(path)?,
        "webp" => decode_webp_frames(path)?,
        _ => return None,
    };

    if frames.len() <= 1 {
        // Single-frame animation = static. Let the static decoder handle it.
        return None;
    }
    Some(convert_frames(frames))
}

fn decode_gif_frames(path: &Path) -> Option<Vec<Frame>> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            log::error!("decode_gif_frames: open failed {}: {e}", path.display());
            return None;
        }
    };
    let decoder = match GifDecoder::new(BufReader::new(file)) {
        Ok(d) => d,
        Err(e) => {
            log::error!("decode_gif_frames: decoder init failed {}: {e}", path.display());
            return None;
        }
    };
    match decoder.into_frames().collect_frames() {
        Ok(f) => {
            log::debug!("decode_gif_frames: {} frames in {}", f.len(), path.display());
            Some(f)
        }
        Err(e) => {
            log::error!("decode_gif_frames: frame collect failed {}: {e}", path.display());
            None
        }
    }
}

fn decode_webp_frames(path: &Path) -> Option<Vec<Frame>> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            log::error!("decode_webp_frames: open failed {}: {e}", path.display());
            return None;
        }
    };
    let decoder = match WebPDecoder::new(BufReader::new(file)) {
        Ok(d) => d,
        Err(e) => {
            log::error!("decode_webp_frames: decoder init failed {}: {e}", path.display());
            return None;
        }
    };
    if !decoder.has_animation() {
        // Static webp — let the static path handle it.
        return None;
    }
    match decoder.into_frames().collect_frames() {
        Ok(f) => {
            log::debug!("decode_webp_frames: {} frames in {}", f.len(), path.display());
            Some(f)
        }
        Err(e) => {
            log::error!("decode_webp_frames: frame collect failed {}: {e}", path.display());
            None
        }
    }
}

fn convert_frames(frames: Vec<Frame>) -> Vec<AnimatedFrame> {
    frames
        .into_iter()
        .map(|f| {
            let delay_ms = {
                let (num, den) = f.delay().numer_denom_ms();
                if den == 0 { 100 } else { (num / den).max(20) }
            };
            let buf = f.into_buffer();
            let (w, h) = buf.dimensions();
            AnimatedFrame {
                rgba: buf.into_raw(),
                width: w,
                height: h,
                delay_ms,
            }
        })
        .collect()
}
