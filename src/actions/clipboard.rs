use gdk4 as gdk;
use gdk4::prelude::*;

pub fn copy_texture_to_clipboard(texture: &gdk::Texture) {
    if let Some(display) = gdk::Display::default() {
        display.clipboard().set_texture(texture);
    }
}

pub fn copy_text_to_clipboard(text: &str) {
    if let Some(display) = gdk::Display::default() {
        let value = text.to_string().to_value();
        let content = gdk::ContentProvider::for_value(&value);
        let _ = display.clipboard().set_content(Some(&content));
    }
}
