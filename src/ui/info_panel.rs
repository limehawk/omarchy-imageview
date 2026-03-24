use gtk4::prelude::*;
use gtk4;
use std::path::Path;

pub struct InfoPanel {
    pub container: gtk4::ScrolledWindow,
    content_box: gtk4::Box,
}

impl InfoPanel {
    pub fn new() -> Self {
        let container = gtk4::ScrolledWindow::new();
        container.add_css_class("info-panel");
        container.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        container.set_size_request(280, -1);

        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        content_box.set_margin_start(12);
        content_box.set_margin_end(12);
        content_box.set_margin_top(8);
        content_box.set_margin_bottom(8);
        container.set_child(Some(&content_box));

        Self {
            container,
            content_box,
        }
    }

    pub fn update(&self, path: Option<&Path>) {
        // Clear existing content
        while let Some(child) = self.content_box.first_child() {
            self.content_box.remove(&child);
        }

        let path = match path {
            Some(p) => p,
            None => return,
        };

        // File info section
        self.add_section("File");
        if let Some(name) = path.file_name() {
            self.add_row("Name", &name.to_string_lossy());
        }
        if let Some(parent) = path.parent() {
            self.add_row("Path", &parent.display().to_string());
        }
        if let Ok(meta) = std::fs::metadata(path) {
            self.add_row("Size", &format_size(meta.len()));
        }

        // Image dimensions — read without full decode
        if let Ok(reader) = image::ImageReader::open(path) {
            if let Ok((w, h)) = reader.into_dimensions() {
                self.add_section("Image");
                self.add_row("Dimensions", &format!("{} \u{00d7} {}", w, h));
            }
        }

        // EXIF data
        if let Ok(file) = std::fs::File::open(path) {
            let mut bufreader = std::io::BufReader::new(file);
            if let Ok(exif) = exif::Reader::new().read_from_container(&mut bufreader) {
                self.add_section("EXIF");
                let fields = [
                    (exif::Tag::Make, "Camera"),
                    (exif::Tag::Model, "Model"),
                    (exif::Tag::LensModel, "Lens"),
                    (exif::Tag::ISOSpeed, "ISO"),
                    (exif::Tag::ExposureTime, "Shutter"),
                    (exif::Tag::FNumber, "Aperture"),
                    (exif::Tag::FocalLength, "Focal Length"),
                    (exif::Tag::DateTimeOriginal, "Date Taken"),
                ];
                for (tag, label) in fields {
                    if let Some(field) = exif.get_field(tag, exif::In::PRIMARY) {
                        self.add_row(label, &field.display_value().to_string());
                    }
                }
            }
        }
    }

    fn add_section(&self, title: &str) {
        let label = gtk4::Label::new(None);
        label.set_markup(&format!("<b>{}</b>", title));
        label.set_xalign(0.0);
        label.set_margin_top(8);
        self.content_box.append(&label);
        self.content_box
            .append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
    }

    fn add_row(&self, key: &str, value: &str) {
        let row = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        let key_label = gtk4::Label::new(Some(key));
        key_label.set_xalign(0.0);
        key_label.add_css_class("info-label");
        let val_label = gtk4::Label::new(Some(value));
        val_label.set_xalign(0.0);
        val_label.set_wrap(true);
        val_label.add_css_class("info-value");
        val_label.set_selectable(true);
        row.append(&key_label);
        row.append(&val_label);
        self.content_box.append(&row);
    }
}

fn format_size(size: u64) -> String {
    let mut s = size as f64;
    for unit in &["B", "KB", "MB", "GB"] {
        if s < 1024.0 {
            return if *unit == "B" {
                format!("{} B", size)
            } else {
                format!("{:.1} {}", s, unit)
            };
        }
        s /= 1024.0;
    }
    format!("{:.1} TB", s)
}
