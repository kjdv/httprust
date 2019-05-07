
use std::net::TcpListener;
mod threadpool;
use threadpool::ThreadPool;

#[cfg(test)]
mod fakestream;

extern crate clap;
extern crate log;
extern crate simple_logger;


mod handler;


fn main() {
    let args = clap::App::new("httprust")
        .author("Klaas de Vries")
        .about("Simple http server")
        .arg(
            clap::Arg::with_name("debug")
                .short("d")
                .long("debug")
                .takes_value(false)
                .help("enable debug logging"),
        )
        .arg(
            clap::Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true)
                .default_value("8080")
                .help("set port to listen on"),
        )
        .get_matches();

    let log_level = {
        if args.is_present("debug") {
            log::Level::Debug
        } else {
            log::Level::Info
        }
    };
    simple_logger::init_with_level(log_level).unwrap();

    let address = format!("127.0.0.1:{}", args.value_of("port").unwrap());

    log::info!("listening on {}", address);

    let listener = TcpListener::bind(address).unwrap();
    let pool = ThreadPool::new(4);

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
