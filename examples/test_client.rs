extern crate log;
extern crate pretty_env_logger;
extern crate reqwest;

fn main() {
    pretty_env_logger::init_timed();

    let args = clap::App::new("httprust")
        .author("Klaas de Vries")
        .about("Simple http test client")
        .about("perform a head request")

        .arg(
            clap::Arg::with_name("URL")
                .required(true)
                .index(1)
                .help("url to perform the request on"),
        )
        .arg(
            clap::Arg::with_name("method")
                .short("m")
                .long("method")
                .takes_value(true)
                .default_value("get")
                .possible_values(&["get", "post"])
                .help("request method"),
        )
        .get_matches();


    let url = args.value_of("URL").unwrap();

    let mut response = reqwest::Client::new()
        .get(url)
        .send().expect("request failure");

    log::info!("{:#?}", response);
    response.copy_to(&mut std::io::stdout()).expect("failed write");
}
