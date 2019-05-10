use super::log;
use std::fs;
use std::path::{Path, PathBuf};
use hyper::{Body, Request, Response, StatusCode};


pub struct Handler {
    root: PathBuf,
}

impl Handler {
    pub fn new(root: &str) -> std::io::Result<Handler> {
        let root = Path::new(root).canonicalize()?;
        log::info!("new handler for root at {:?}", root);

        Ok(Handler{
            root: root,
        })
    }

    pub fn handle(&self, request: Request<Body>) -> Response<Body> {
        log::info!(
            "handling {} request for {}",
            request.method(),
            request.uri()
        );
        log::debug!("{:#?}", request);

        let path = String::from(request.uri().path());
        let path = path.trim_start_matches("/");
        let path = self.root.join(path);
        log::debug!("serving {:?}", path);

        match fs::read_to_string(path) {
            Ok(content) => Response::new(Body::from(content)),
            Err(e) => {
                log::warn!("{}", e);
                let response = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("resource not found"))
                    .expect("invalid response");
                response
            }
        }
    }
}
