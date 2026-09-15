use omarchy_imageview::core::image_loader::{
    load_image, svg_exceeds_cap, svg_intrinsic_size, MAX_SVG_EDGE,
};

#[test]
fn reads_width_and_height() {
    let svg = r#"<svg width="1024" height="768" xmlns="http://www.w3.org/2000/svg"></svg>"#;
    assert_eq!(svg_intrinsic_size(svg), Some((1024, 768)));
}

#[test]
fn reads_view_box_when_width_missing() {
    let svg = r#"<svg viewBox="0 0 320 240" xmlns="http://www.w3.org/2000/svg"></svg>"#;
    assert_eq!(svg_intrinsic_size(svg), Some((320, 240)));
}

#[test]
fn ceils_fractional_width() {
    let svg = r#"<svg width="633631.5" height="543913.75" viewBox="0 0 633631.5 543913.75"></svg>"#;
    assert_eq!(svg_intrinsic_size(svg), Some((633632, 543914)));
}

#[test]
fn flags_huge_aseprite_export() {
    assert!(svg_exceeds_cap(633632, 543914));
    assert!(!svg_exceeds_cap(1024, 1024));
    assert!(!svg_exceeds_cap(MAX_SVG_EDGE, MAX_SVG_EDGE));
    assert!(svg_exceeds_cap(MAX_SVG_EDGE + 1, 1));
}

#[test]
fn load_image_skips_huge_svg() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("huge.svg");
    std::fs::write(
        &path,
        r#"<svg width="633631.5" height="543913.75" xmlns="http://www.w3.org/2000/svg">
           <rect width="1" height="1" fill="black"/>
           </svg>"#,
    )
    .unwrap();
    assert!(load_image(&path).is_none());
}

#[test]
fn load_image_decodes_small_svg() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tiny.svg");
    std::fs::write(
        &path,
        concat!(
            "<svg xmlns='http://www.w3.org/2000/svg' width='10' height='10'>",
            "<rect width='10' height='10' fill='red'/></svg>",
        ),
    )
    .unwrap();
    let img = load_image(&path).expect("small svg should decode");
    assert_eq!(img.width(), 10);
    assert_eq!(img.height(), 10);
}
