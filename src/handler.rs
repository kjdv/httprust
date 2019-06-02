extern crate path_abs;

use path_abs::{PathDir, PathFile};
use super::log;
use futures::{future, Future};
use hyper::{Body, Request, Response, StatusCode, Method, HeaderMap, header};
use hyper::header::HeaderValue;
use crate::async_stream::AsyncStream;
use crate::meta_info::*;
use crate::compressed_read::*;

type ResponseFuture = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;

pub struct Handler {
    root: PathDir,
}

impl Handler {
    pub fn new(root: &str) -> std::io::Result<Handler> {
        let root = PathDir::new(root)?.canonicalize()?;
        log::info!("new handler for root at {:?}", root);

        Ok(Handler{
            root,
        })
    }

    pub fn handle(&self, request: Request<Body>) -> ResponseFuture {
        log::info!(
            "handling {} request for {}",
            request.method(),
            request.uri()
        );
        log::debug!("{:#?}", request);

        match *request.method() {
            Method::GET => {},
            Method::HEAD => {},
            _ => {
                log::debug!("unsuppored method");
                return direct_response(StatusCode::METHOD_NOT_ALLOWED);
            }
        }

        let path = String::from(request.uri().path());
        let path = path.trim_start_matches('/');
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

        let path = match PathFile::new(path) {
            Ok(p) => p,
            Err(e) => {
                log::info!("file does not exist: {}", e);
                return direct_response(StatusCode::NOT_FOUND);
            }
        };

        serve_file(path, request)
    }
}

fn should_compress(m: &Mime, headers: &HeaderMap<HeaderValue>) -> bool {
    if is_compressable(m) {
        log::debug!("{} is eligable for compression", m);

        match headers.get(header::ACCEPT_ENCODING) {
            Some(ae) => {
                match ae.to_str() {
                    Ok(aes) => {
                        let s = String::from(aes);
                        s.split(',').any(|v| v == "gzip")
                    },
                    Err(_) => false,
                }
            },
            None => false,
        }
    } else {
        log::debug!("{} is not eligable for compression", m);
        false
    }
}

fn serve_file(path: PathFile, request: Request<Body>) -> ResponseFuture {
    log::debug!("serving {:?}", path);

    let mut builder = Response::builder();
    let mut use_gzip = false;
    if let Some(mime) = sniff_mime(path.as_os_str()) {
        builder.header(header::CONTENT_TYPE, mime.to_string());

        if should_compress(&mime, request.headers()) {
            log::debug!("compressing {:?}", path);
            builder.header(header::CONTENT_ENCODING, "gzip");
            use_gzip = true;
        }
    }


    let fut = tokio::fs::file::File::open(path)
        .and_then(move |file| {
            let body = match *request.method() {
                Method::HEAD => Body::empty(),
                Method::GET => {
                    if use_gzip {
                        let file = CompressedRead::new(file);
                        let stream = AsyncStream::new(file);
                        Body::wrap_stream(stream)
                    } else {
                        let stream = AsyncStream::new(file);
                        Body::wrap_stream(stream)
                    }
                },
                _ => panic!("unreachable!"),
            };

            Ok(builder
                .status(StatusCode::OK)
                .body(body)
                .unwrap())
        })
        .or_else(|e| {
            log::warn!("error serving file: {}", e);
            Ok(raw_direct_response(StatusCode::NOT_FOUND))
        });
    Box::new(fut)
}

fn raw_direct_response(code: StatusCode) -> Response<Body> {
    Response::builder()
        .status(code)
        .body(Body::from(code.canonical_reason().unwrap_or("")))
        .unwrap()
}

fn direct_response(code: StatusCode) -> ResponseFuture {
    let response = raw_direct_response(code);
    Box::new(future::ok(response))
}

#[cfg(test)]
mod tests {
    extern crate tokio;
    use tokio::runtime::current_thread;

    use super::*;

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

        current_thread::Runtime::new()
            .expect("new runtime")
            .spawn(response_future)
            .run()
            .expect("run runtime");
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
}
