use reqwest;
use std::sync::Once;

pub use reqwest::{Client, Error, Response, StatusCode};

pub const PORT: u16 = 2950;
pub const TLS_PORT: u16 = PORT + 1;
pub const ADDRESS: &str = "localhost";

pub fn server() {
    static SERVER: Once = Once::new();
    SERVER.call_once(|| {
        let cargo_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR to be set");
        let root = std::path::PathBuf::from(cargo_dir)
            .join("tests")
            .join("sample_root");

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let cfg = httprust::Config {
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

pub fn tls_server() {
    static SERVER: Once = Once::new();
    SERVER.call_once(|| {
        let cargo_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR to be set");
        let root = std::path::PathBuf::from(cargo_dir.clone())
            .join("tests")
            .join("sample_root");
        let tls_config = std::path::PathBuf::from(cargo_dir)
            .join("tests")
            .join("sample_tls");
        let cert_file = tls_config.join("httprust-test-cert.pem");
        let key_file = tls_config.join("httprust-test-key.pem");

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let cfg = httprust::Config {
                port: TLS_PORT,
                local_only: true,
                root: String::from(root.to_str().unwrap()),
                tls: Some(httprust::TlsConfig {
                    certificate_file: String::from(cert_file.to_str().unwrap()),
                    private_key_file: String::from(key_file.to_str().unwrap()),
                }),
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

pub fn make_tls_uri(resource: &str) -> String {
    format!("https://{}:{}/{}", ADDRESS, TLS_PORT, resource)
}

pub fn get(resource: &str) -> Result<Response, Error> {
    let uri = make_uri(resource);

    Client::new().get(uri.as_str()).send()
}
