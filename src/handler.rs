extern crate path_abs;

use path_abs::PathDir;
use super::log;
use std::fs;
use futures::{future, Future};
use hyper::{Body, Request, Response, StatusCode, Method};


type ResponseFuture = Box<Future<Item=Response<Body>, Error=std::io::Error> + Send>;

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

    pub fn handle(&self, request: Request<Body>) -> ResponseFuture {
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
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(content))
                    .expect("invalid response");
                Box::new(future::ok(response))
            }
            Err(e) => {
                log::warn!("{}", e);
                direct_response(StatusCode::NOT_FOUND)
            }
        }
    }
}

fn direct_response(code: StatusCode) -> ResponseFuture {
    let response = Response::builder()
        .status(code)
        .body(Body::from(code.canonical_reason().unwrap_or("")))
        .unwrap();
    Box::new(future::ok(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::Stream;

    fn make_handler() -> Handler {
        let cargo_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR to be set");
        let root = std::path::PathBuf::from(cargo_dir)
            .join("tests")
            .join("sample_root");

        Handler::new(root.to_str().unwrap()).expect("making handler")
    }

    fn handle<F>(request: Request<Body>, check: F)
        where F: FnOnce(Response<Body>) + Send + 'static {
        let handler = make_handler();

        let response_future = handler.handle(request)
            .map(check)
            .map_err(|e| {
                panic!("error checking: {}", e);
            });
        hyper::rt::run(response_future);
    }

    fn check_code_for_resource(resource: &str, expect: StatusCode) {
        let uri = format!("http://something/{}", resource);
        let request = Request::builder()
            .uri(uri)
            .body(Body::from(""))
            .unwrap();
            handle(request, move |res| {
                assert_eq!(expect, res.status());
            });
    }

    #[test]
    fn get_content() {
        let request = Request::builder()
                .uri("http://something/hello.txt")
                .body(Body::from(""))
                .unwrap();
        handle(request, |res| {
            assert_eq!(StatusCode::OK, res.status());

            res.into_body()
                .take(1)
                .for_each(|chunk| {
                    let actual = std::str::from_utf8(chunk.as_ref())
                        .expect("valid utf-8");
                    assert_eq!("hello!\n", actual);
                    Ok(())
                }).poll().expect("check");
        });
    }

    #[test]
    fn not_found() {
        check_code_for_resource("no_such_thing", StatusCode::NOT_FOUND);
    }

    #[test]
    fn no_directory_traversal_allowed() {
        check_code_for_resource("../requests.rs", StatusCode::FORBIDDEN);
    }

    #[test]
    fn no_directory_traversal_allowed_for_non_existant() {
        // subtlety: if we have a traversal for a file that does not actually exist, we want this
        // reported as forbidden instead of not found: don't leak more information than you have to
        check_code_for_resource("../no_such_thing", StatusCode::FORBIDDEN);
    }

    #[test]
    fn no_post_on_static_file() {
        let request = Request::builder()
            .uri("http://something/index.html")
            .method("POST")
            .body(Body::from(""))
            .unwrap();

        handle(request, |res| {
            assert_eq!(StatusCode::METHOD_NOT_ALLOWED, res.status());
        });
    }

    #[test]
    fn head_returns_no_data() {
        let request = Request::builder()
            .uri("http://something/index.html")
            .method("HEAD")
            .body(Body::from(""))
            .unwrap();

        handle(request, |res| {
            assert_eq!(StatusCode::OK, res.status());
            res.into_body()
                .fold(Vec::new(), |mut acc, chunk| {
                    acc.extend_from_slice(&*chunk);
                    future::ok::<_, hyper::Error>(acc)
                })
                .and_then(|v| {
                    assert!(v.is_empty());
                    Ok(())
                }).poll().expect("ready");
        });
    }
}
