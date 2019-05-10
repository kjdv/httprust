use super::log;
use std::fs;
use std::path::{Path, PathBuf};
use hyper::{Body, Request, Response, StatusCode, Method};


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

        match request.method() {
            &Method::GET => {},
            &Method::HEAD => {},
            _ => {
                log::debug!("unsuppored method");
                return method_not_allowed();
            }
        }

        let path = String::from(request.uri().path());
        let path = path.trim_start_matches("/");

        let path = match self.root.join(path).canonicalize() {
            Ok(p) => {
                if !p.starts_with(self.root.to_owned()) {
                    log::warn!("attempted directory traversal: {:?}", p);
                    return forbidden();
                }
                p
            }
            Err(e) => {
                log::warn!("canonicalization error {}", e);
                if Path::new(path).components().any(|c| c.as_os_str() == "..") {
                    log::warn!("attempted directory traversal: {:?}", path);
                    return forbidden();
                }
                return not_found();
            }
        };
        log::debug!("serving {:?}", path);

        match fs::read_to_string(path) {
            Ok(content) => Response::new(Body::from(content)),
            Err(e) => {
                log::warn!("{}", e);
                not_found()
            }
        }
    }
}

fn not_found() -> Response<Body> {
    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("resource not found"))
        .expect("invalid response");
    response
}

fn forbidden() -> Response<Body> {
    let response = Response::builder()
        .status(StatusCode::FORBIDDEN)
        .body(Body::from("forbidden"))
        .expect("invalid response");
    response
}

fn method_not_allowed() -> Response<Body> {
    let response = Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::from("method not  allowed"))
        .expect("invalid response");
    response
}
