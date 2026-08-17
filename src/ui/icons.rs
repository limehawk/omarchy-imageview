//! Toolbar icons from [Remix Icon](https://remixicon.com/) (Line).
//! Remix Icon License v1.0 — see `assets/icons/LICENSE.md`.
//! Tinted to the Omarchy foreground and rasterized at the widget scale.

use gdk4 as gdk;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::gio;
use gtk4::prelude::*;
use gtk4::{self, glib};

const SIZE: i32 = 24;
const SOURCE_STROKE: &str = "currentColor";

pub struct Icon {
    pub svg: &'static str,
}

pub const SAVE: Icon = Icon {
    svg: include_str!("../../assets/icons/save-line.svg"),
};
pub const ROTATE: Icon = Icon {
    svg: include_str!("../../assets/icons/reset-right-line.svg"),
};
pub const COPY: Icon = Icon {
    svg: include_str!("../../assets/icons/file-copy-line.svg"),
};
pub const TRASH: Icon = Icon {
    svg: include_str!("../../assets/icons/delete-bin-line.svg"),
};
pub const FILMSTRIP: Icon = Icon {
    svg: include_str!("../../assets/icons/film-line.svg"),
};
pub const SORT: Icon = Icon {
    svg: include_str!("../../assets/icons/sort-desc.svg"),
};
pub const INFO: Icon = Icon {
    svg: include_str!("../../assets/icons/information-line.svg"),
};

fn foreground_hex() -> String {
    let [r, g, b, _] = super::theme::foreground_rgba();
    format!("#{r:02X}{g:02X}{b:02X}")
}

pub fn image(icon: &'static Icon) -> gtk4::Image {
    let img = gtk4::Image::new();
    img.set_pixel_size(SIZE);
    img.add_css_class("toolbar-icon");
    paint(&img, icon);
    {
        let img = img.clone();
        img.connect_realize(move |w| paint(w, icon));
    }
    {
        let img = img.clone();
        img.connect_notify_local(Some("scale-factor"), move |w, _| paint(w, icon));
    }
    img
}

pub fn button(name: &str, icon: &'static Icon, tooltip: &str) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.set_tooltip_text(Some(tooltip));
    btn.set_has_frame(false);
    btn.set_valign(gtk4::Align::Center);
    btn.add_css_class("pixel-btn");
    btn.set_child(Some(&image(icon)));
    let _ = name;
    btn
}

fn paint(img: &gtk4::Image, icon: &Icon) {
    let scale = img.scale_factor().max(1);
    let px = SIZE * scale;
    let svg = icon.svg.replace(SOURCE_STROKE, &foreground_hex());
    let bytes = glib::Bytes::from(svg.as_bytes());
    let stream = gio::MemoryInputStream::from_bytes(&bytes);
    let Ok(pixbuf) = Pixbuf::from_stream_at_scale(&stream, px, px, true, gio::Cancellable::NONE)
    else {
        return;
    };
    img.set_paintable(Some(&gdk::Texture::for_pixbuf(&pixbuf)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remix_are_filled_svgs() {
        for icon in [&SAVE, &ROTATE, &COPY, &TRASH, &FILMSTRIP, &INFO, &SORT] {
            assert!(icon.svg.contains("viewBox=\"0 0 24 24\""));
            assert!(icon.svg.contains(SOURCE_STROKE));
        }
    }
}
