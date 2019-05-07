extern crate log;
extern crate simple_logger;
extern crate hyper;

use hyper::rt::{Future, Stream};

use std::io::Write;

fn main() {
    let args = clap::App::new("httprust")
        .author("Klaas de Vries")
        .about("Simple http test client")
        .about("perform a head request")
        .arg(clap::Arg::with_name("URL")
            .required(true)
            .index(1)
            .help("url to perform the request on"))
        .arg(clap::Arg::with_name("method")
            .short("m")
            .long("method")
            .takes_value(true)
            .default_value("get")
            .possible_values(&["get", "post"])
            .help("request method"))
        .arg(
            clap::Arg::with_name("debug")
                .short("d")
                .long("debug")
                .takes_value(false)
                .help("enable debug logging"),
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

    let method = args.value_of("method").unwrap();
    let url = args.value_of("URL").unwrap().parse().unwrap();

    hyper::rt::run(fetch(url, method));
}

fn fetch(uri: hyper::Uri, method: &str) -> impl Future<Item=(), Error=()> {
    let client = hyper::Client::new();

    let response_future;

    if method == "get" {
        log::debug!("get request");
        response_future = client.get(uri);
    } else if method == "post" {
        log::error!("post requests not implemented yet");
        panic!("not implemented");
    } else {
        panic!("bad method");
    }

    response_future
        .and_then(|res| {
            log::debug!("{:#?}", res);
            res
                .into_body()
                .for_each(|chunk| {
                    std::io::stdout()
                        .write_all(&chunk)
                        .map_err(|e| {
                            log::error!("write failure: {}", e);
                            panic!("write failure");
                        })
                })
        })
        .map(|_| {
            log::debug!("done");
        })
        .map_err(|e| {
            log::error!("request failure {}", e);
        })
}
