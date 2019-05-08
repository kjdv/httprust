use std::sync::{Once, ONCE_INIT};
use hyper;
use httprust;
use hyper::rt::Future;
use tokio::runtime::current_thread;

static SERVER: Once = ONCE_INIT;

const PORT: u16 = 2950;
const ADDRESS: &str = "http://127.0.0.1";

pub type Response = hyper::Response<hyper::Body>;

pub fn server() {
    SERVER.call_once(|| {
        std::thread::spawn(|| {
            httprust::run(httprust::Config{port: PORT, local_only: true});
        });
    });
}

pub fn get<F>(resource: &str, on_response: F) where F: FnOnce(Response) + 'static {
    let uri = format!("{}:{}/{}", ADDRESS, PORT, resource);
    let client = hyper::Client::new()
        .get(uri.parse::<hyper::Uri>().expect("uri"))
        .and_then(|res| {
            on_response(res);
            Ok(())
        })
        .map_err(|e| {
            panic!("error {}", e);
        });

    current_thread::Runtime::new()
        .expect("new runtime")
        .spawn(client)
        .run()
        .expect("run");
}

pub fn is_ok(res: Response) {
    assert_eq!(hyper::StatusCode::OK, res.status());
}
