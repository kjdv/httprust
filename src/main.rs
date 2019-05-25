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
        .arg(
            clap::Arg::with_name("tls")
                .short("t")
                .long("tls")
                .takes_value(true)
                .help("use tls with the identity file and password provided (use the format 'identity.p12,mypass'")
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
        tls: parse_tls(args.value_of("tls")),
    };
    httprust::run(cfg);
}

fn parse_tls(s: Option<&str>) -> Option<httprust::TlsConfig> {
    s.map(|s| {
        let s = String::from(s);
        let mut it = s.split(',');
        let filename = it.next().map(String::from).expect("no filename given for tls argument");
        let password = it.next().map(String::from);

        if it.next().is_some() {
            panic!("more than 2 items given for tls argument");
        }

        httprust::TlsConfig{
            identity: filename,
            password: password,
        }
    })
}


#[cfg(test)]
mod tests {
    #[test]
    fn parse_tls() {
        let cases = [
            (None, None),
            (Some("identity.p12"), Some(("identity.p12", None))),
            (Some("identity.p12,pass"), Some(("identity.p12", Some("pass")))),
        ];

        for (input, expect) in cases.into_iter() {
            let actual = super::parse_tls(*input);

            match expect {
                None => assert!(actual.is_none()),
                Some((i, p)) => {
                    let actual = actual.expect("some");
                    assert_eq!(String::from(*i), actual.identity);
                    assert_eq!(p.map(String::from), actual.password);
                }
            }
        }
    }

    #[test]
    #[should_panic]
    fn parse_tls_too_many_args() {
        super::parse_tls(Some("a,b,c"));
    }
}