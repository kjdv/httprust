extern crate clap;
extern crate simple_logger;

use httprust;


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
        .arg(
            clap::Arg::with_name("local_only")
                .short("l")
                .long("local-only")
                .takes_value(false)
                .help("only open for local connections")
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
    let local_only = args.is_present("local_only");

    let mut app = httprust::App::new(port, local_only);
    app.run().expect("failed to start");
}
