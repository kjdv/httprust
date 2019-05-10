extern crate clap;
extern crate pretty_env_logger;

use httprust;


fn main() {
    pretty_env_logger::init_timed();

    let cwd = std::env::current_dir().expect("get cwd");
    let cwd = cwd.to_str().unwrap();

    let args = clap::App::new("httprust")
        .author("Klaas de Vries")
        .about("Simple http server")
        .arg(
            clap::Arg::with_name("root")
                .short("r")
                .long("root")
                .takes_value(true)
                .default_value(cwd)
                .help("root of the directory to serve")
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

    let cfg = httprust::Config{
        port: args
            .value_of("port")
            .unwrap()
            .parse::<u16>()
            .expect("invalid port number"),
        local_only: args.is_present("local_only"),
        root: args.value_of("root").unwrap().to_string(),
    };
    httprust::run(cfg);
}
