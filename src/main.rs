
use std::net::TcpListener;
mod threadpool;
use threadpool::ThreadPool;

#[cfg(test)]
mod fakestream;


extern crate log;
extern crate simple_logger;

mod handler;


#[cfg(debug_assertions)]
fn init_logging() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
}
#[cfg(not(debug_assertions))]
fn init_logging() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
}

fn main() {
    init_logging();

    let address = "127.0.0.1:7878";

    let listener = TcpListener::bind(address).unwrap();
    let pool = ThreadPool::new(4);

    log::info!("listening on {}", address);

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                pool.execute(|| {
                    handler::handle(s).unwrap_or_else(|e| log::error!("failure: {}", e));
                })
                .unwrap();
            }
            Err(error) => {
                log::error!("error with incoming stream: {:?}", error);
            }
        }
    }
}
