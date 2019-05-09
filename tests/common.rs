use std::sync::{Once, ONCE_INIT};
use hyper;
use log;
use httprust;
use hyper::rt::Future;
use tokio::runtime::current_thread;

static SERVER: Once = ONCE_INIT;

const PORT: u16 = 2950;
const ADDRESS: &str = "127.0.0.1";

pub type Response = hyper::Response<hyper::Body>;

pub fn server() {
    SERVER.call_once(|| {
        std::thread::spawn(|| {
            httprust::run(httprust::Config{port: PORT, local_only: true});
        });

        busy_wait(|| try_connect(ADDRESS, PORT), 1).expect("connect");
    });
}

pub fn get<F>(resource: &str, on_response: F) where F: FnOnce(Response) + 'static {
    let uri = format!("http://{}:{}/{}", ADDRESS, PORT, resource);

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
