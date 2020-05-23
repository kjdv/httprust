extern crate mime;
extern crate mime_guess;

pub use mime_guess::Mime;
use std::ffi::OsStr;

pub fn sniff_mime(path: &OsStr) -> Option<Mime> {
    mime_guess::from_path(path).first()
}

pub fn is_compressable(m: &Mime) -> bool {
    if m.type_() == mime::TEXT {
        return true;
    }

    match m.subtype() {
        mime::JAVASCRIPT => true,
        mime::JSON => true,
        mime::TEXT => true,
        mime::XML => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniff_mime() {
        let cases = [
            ("f.unknown", None),
            ("f", None),
            ("f.txt", Some(mime::TEXT_PLAIN)),
            ("f.pdf", Some(mime::APPLICATION_PDF)),
            ("f.jpg", Some(mime::IMAGE_JPEG)),
            ("f.jpeg", Some(mime::IMAGE_JPEG)),
        ];

        for (filename, expect) in &cases {
            let actual = super::sniff_mime(OsStr::new(filename));
            assert_eq!(*expect, actual.map(|v| v.clone()));
        }
    }

    #[test]
    fn is_compressable() {
        let cases = [
            ("f.txt", true),
            ("f.png", false),
            ("f.html", true),
            ("f.json", true),
        ];

        for (filename, expect) in &cases {
            let mime = super::sniff_mime(OsStr::new(filename)).expect("valid mime");
            let actual = super::is_compressable(&mime);
            assert_eq!(*expect, actual);
        }
    }
}
