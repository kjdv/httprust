extern crate mime;

pub use mime::Mime;
use path_abs::PathAbs;
use std::collections::HashMap;
use std::ffi::OsStr;


pub fn sniff_mime(path: &str) -> Option<&Mime> {
    lazy_static! {
        static ref MAPPING: HashMap<&'static OsStr, Mime> = {
            let mut map = HashMap::new();

            // extend when needed
            map.insert(OsStr::new("js"),   mime::APPLICATION_JAVASCRIPT);
            map.insert(OsStr::new("json"), mime::APPLICATION_JSON);
            map.insert(OsStr::new("pdf"),  mime::APPLICATION_PDF);
            map.insert(OsStr::new("png"),  mime::IMAGE_PNG);
            map.insert(OsStr::new("jpg"),  mime::IMAGE_JPEG);
            map.insert(OsStr::new("jpeg"), mime::IMAGE_JPEG);
            map.insert(OsStr::new("txt"),  mime::TEXT_PLAIN);
            map.insert(OsStr::new("css"),  mime::TEXT_CSS);
            map.insert(OsStr::new("csv"),  mime::TEXT_CSV);
            map.insert(OsStr::new("html"), mime::TEXT_HTML);
            map.insert(OsStr::new("text"), mime::TEXT_PLAIN);
            map.insert(OsStr::new("xml"),  mime::TEXT_XML);
        
            map
        };
    };

    PathAbs::new(path)
        .map(|p| {
            match p.extension() {
                Some(e) => MAPPING.get(e),
                None => None,
            }
        })
        .unwrap_or(None)
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
            let actual = super::sniff_mime(filename);
            assert_eq!(*expect, actual.map(|v| v.clone()));
        }
    }
}
