use std::sync::{Once, ONCE_INIT};
use reqwest;

pub use reqwest::{Client, Response, Error, StatusCode};

pub const PORT: u16 = 2950;
pub const ADDRESS: &str = "127.0.0.1";


pub fn server() {
    static SERVER: Once = ONCE_INIT;
    SERVER.call_once(|| {
        let cargo_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR to be set");
        let root = std::path::PathBuf::from(cargo_dir)
            .join("tests")
            .join("sample_root");

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let cfg = httprust::Config{
                port: PORT,
                local_only: true,
                root: String::from(root.to_str().unwrap()),
                tls: None,
            };
            httprust::run_notify(cfg, move || {
                tx.send(()).expect("no notify readyness");
            });
        });

        rx.recv().expect("to be ready");
    });
}

pub fn make_uri(resource: &str) -> String {
    format!("http://{}:{}/{}", ADDRESS, PORT, resource)
}

pub fn get(resource: &str) -> Result<Response, Error> {
    let uri = make_uri(resource);

    Client::new()
        .get(uri.as_str())
        .send()
}
