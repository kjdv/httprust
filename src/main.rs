extern crate clap;
extern crate hyper;
extern crate log;
extern crate simple_logger;


use hyper::rt::Future;

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

    let port = args
        .value_of("port")
        .unwrap()
        .parse::<u16>()
        .expect("invalid port number");
    let address = ([127, 0, 0, 1], port).into();

    let server = hyper::Server::bind(&address)
        .serve(|| hyper::service::service_fn_ok(handler::handle))
        .map_err(|e| {
            log::error!("server error {}", e);
        });

    log::info!("listening on {:?}", address);

    hyper::rt::run(server);

}
