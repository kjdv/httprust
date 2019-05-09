
use super::log;

use hyper::{Body, Request, Response};

pub fn handle(request: Request<Body>) -> Response<Body> {
    log::info!(
        "handling {} request for {}",
        request.method(),
        request.uri()
    );
    log::debug!("{:#?}", request);

    Response::new(Body::from("hello!\n"))
}
