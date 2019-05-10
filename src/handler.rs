use super::log;
use std::path::{Path, PathBuf};
use hyper::{Body, Request, Response};


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

        Response::new(Body::from("hello!\n"))
    }
}
