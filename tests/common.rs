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

        std::thread::spawn(move || {
            let cfg = httprust::Config{
                port: PORT,
                local_only: true,
                root: String::from(root.to_str().unwrap()),
            };
            httprust::run(cfg);
        });

        busy_wait(|| try_connect(ADDRESS, PORT), 1).expect("connect");
    });
}

pub fn make_uri(resource: &str) -> String {
    format!("http://{}:{}/{}", ADDRESS, PORT, resource)
}

pub fn head(resource: &str) -> Result<Response, Error> {
    let uri = make_uri(resource);

    Client::new()
        .head(uri.as_str())
        .send()
}

pub fn get(resource: &str) -> Result<Response, Error> {
    let uri = make_uri(resource);

    Client::new()
        .get(uri.as_str())
        .send()
}

pub fn post(resource: &str, body: &'static [u8]) -> Result<Response, Error> {
    let uri = make_uri(resource);

    Client::new()
        .post(uri.as_str())
        .body(body)
        .send()
}


fn try_connect(address: &str, port: u16) -> bool {
    let endpoint = format!("{}:{}", address, port);
    log::debug!("trying to connect to {}", endpoint);

    match std::net::TcpStream::connect(endpoint) {
        Ok(_) => {
            log::debug!("succesfull connection");
            true
        },
        Err(_) => false,
    }
}

fn busy_wait<F>(predicate: F, timeout_s: u64) -> Result<(), &'static str>
    where F: Fn() -> bool {
    use std::time::*;

    let now = SystemTime::now();
    let timeout = Duration::new(timeout_s, 0);

    loop {
        if predicate() {
            return Ok(())
        }

        match now.elapsed() {
            Ok(e) => {
                if e >= timeout {
                    return Err("timeout");
                }
                std::thread::yield_now();
            },
            Err(_) => {
                return Err("Error tracking time");
            }
        }
    }
}