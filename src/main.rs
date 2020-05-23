extern crate clap;
extern crate path_abs;
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
            clap::Arg::with_name("root_directory")
                .short("r")
                .long("root")
                .takes_value(true)
                .default_value(cwd)
                .validator(validate_directory)
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
        .arg(
            clap::Arg::with_name("certificate_file")
                .short("c")
                .long("cert")
                .takes_value(true)
                .requires("private_key_file")
                .validator(validate_file)
                .help("when provided, this will be a https server. The proviced certifice will be used")
        )
        .arg(
            clap::Arg::with_name("private_key_file")
                .short("k")
                .long("key")
                .takes_value(true)
                .requires("certificate_file")
                .validator(validate_file)
                .help("needed in combination with --cert, this specifies the file containing the private key")
        )
        .get_matches();

    let cfg = httprust::Config {
        port: args
            .value_of("port")
            .unwrap()
            .parse::<u16>()
            .expect("invalid port number"),
        local_only: args.is_present("local_only"),
        root: args.value_of("root_directory").unwrap().to_string(),
        tls: args
            .value_of("certificate_file")
            .map(|cf| httprust::TlsConfig {
                certificate_file: cf.to_string(),
                private_key_file: args.value_of("private_key_file").unwrap().to_string(),
            }),
    };
    httprust::run(cfg);
}

fn validate_directory(d: String) -> Result<(), String> {
    match path_abs::PathDir::new(d) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{}", e)),
    }
}

fn validate_file(f: String) -> Result<(), String> {
    match path_abs::PathFile::new(f) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{}", e)),
    }
}
