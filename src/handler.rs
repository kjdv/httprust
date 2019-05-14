extern crate path_abs;

use path_abs::PathDir;
use super::log;
use std::fs;
use hyper::{Body, Request, Response, StatusCode, Method};


pub struct Handler {
    root: PathDir,
}

impl Handler {
    pub fn new(root: &str) -> std::io::Result<Handler> {
        let root = PathDir::new(root)?.canonicalize()?;
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

        match request.method() {
            &Method::GET => {},
            &Method::HEAD => {},
            _ => {
                log::debug!("unsuppored method");
                return direct_response(StatusCode::METHOD_NOT_ALLOWED);
            }
        }

        let path = String::from(request.uri().path());
        let path = path.trim_start_matches("/");
        let path = match self.root.join(path).absolute() {
            Ok(p) => p,
            Err(e) => {
                log::warn!("failed to absolute: {}", e);
                return direct_response(StatusCode::BAD_REQUEST);
            }
        };

        if !path.as_path().starts_with(self.root.to_owned()) {
            log::warn!("attempted directory traversal: {:?}", path);
            return direct_response(StatusCode::FORBIDDEN);
        }

        log::debug!("serving {:?}", path);

        match fs::read_to_string(path) {
            Ok(content) => {
                let content = match request.method() {
                    &Method::GET => {content},
                    &Method::HEAD => {String::from("")},
                    _ => panic!("should have been caught earlier"),
                };
                Response::new(Body::from(content))
            }
            Err(e) => {
                log::warn!("{}", e);
                direct_response(StatusCode::NOT_FOUND)
            }
        }
    }
}

fn direct_response(code: StatusCode) -> Response<Body> {
    let response = Response::builder()
        .status(code)
        .body(Body::from(code.canonical_reason().unwrap_or("")))
        .unwrap();
    response
}
