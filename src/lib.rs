extern crate hyper;
extern crate log;
extern crate futures;
extern crate tokio;
extern crate tokio_signal;


use std::sync::Arc;
use hyper::rt::{self, Future, Stream};
use futures::sync::oneshot::{Sender, channel};

mod meta_info;
mod async_stream;
mod compressed_read;
mod handler;
mod tls;

#[derive(Debug)]
pub struct TlsConfig {
    pub certificate_file: String,
    pub private_key_file: String,
}

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub local_only: bool,
    pub root: String,
    pub tls: Option<TlsConfig>,
}

pub fn run_notify<F>(cfg: Config, notify: F)
    where F: FnOnce() + Send + 'static {
    log::info!("starting server with configuration {:#?}", cfg);

    rt::run(rt::lazy(move || {
        if cfg.tls.is_some() {
            let (server, stopper) = make_tls_server(cfg);
            let signal_handler = make_signal_handler(stopper);

            rt::spawn(signal_handler);
            rt::spawn(server);
        } else {
            let (server, stopper) = make_plain_server(cfg);
            let signal_handler = make_signal_handler(stopper);

            rt::spawn(signal_handler);
            rt::spawn(server);
        }

        log::info!("ready to serve");
        notify();

        Ok(())
    }));

    log::info!("done");

}

pub fn run(cfg: Config) {
    run_notify(cfg, || {});
}

fn make_plain_server(cfg: Config) -> (impl Future<Item = (), Error = ()>, Sender<()>) {
    log::warn!("running insecure http (not s) server");

    let address = {
        if cfg.local_only {
            [127, 0, 0, 1]
        } else {
            [0, 0, 0, 0]
        }
    };
    let address = (address, cfg.port).into();

    let handle = handler::Handler::new(cfg.root.as_str())
        .map_err(|e| {
            log::error!("error creating handler {}", e);
            panic!("create handler");
        }).unwrap();

    let handle = Arc::new(handle);

    let handle = move || {
        let this_handler = handle.clone();
        hyper::service::service_fn(move |req| {
            this_handler.handle(req)
        })
    };

    let server = hyper::Server::bind(&address)
        .serve(handle);

    log::info!("listening on {:?}", address);

    let (tx, rx) = channel::<()>();
    let server = server
        .with_graceful_shutdown(rx)
        .map_err(|e| { log::error!("server error {}", e)});

    (server, tx)
}

// todo: plenty of duplicated code with make_plain_server, but heavy use of generics make it very hard to
//       create meaningful return types
fn make_tls_server(cfg: Config) -> (impl Future<Item = (), Error = ()>, Sender<()>) {
    log::info!("running https server");

    let address = {
        if cfg.local_only {
            [127, 0, 0, 1]
        } else {
            [0, 0, 0, 0]
        }
    };
    let address = (address, cfg.port).into();

    let handle = handler::Handler::new(cfg.root.as_str())
        .map_err(|e| {
            log::error!("error creating handler {}", e);
            panic!("create handler");
        }).unwrap();

    let handle = Arc::new(handle);

    let handle = move || {
        let this_handler = handle.clone();
        hyper::service::service_fn(move |req| {
            this_handler.handle(req)
        })
    };

    let cfg = tls::configure_tls(cfg.tls.unwrap()).unwrap();

    let server = tls::make_server(address, cfg).unwrap()
        .serve(handle);

    log::info!("listening on {:?}", address);

    let (tx, rx) = channel::<()>();
    let server = server
        .with_graceful_shutdown(rx)
        .map_err(|e| { log::error!("server error {}", e)});

    (server, tx)
}


fn make_signal_handler(stopper: Sender<()>) -> impl Future<Item = (), Error = ()> {
    use tokio_signal::unix::{Signal, SIGINT, SIGTERM};

    let sigint = Signal::new(SIGINT).flatten_stream();
    let sigterm = Signal::new(SIGTERM).flatten_stream();
    let stream = sigint.select(sigterm);

    stream.into_future()
        .and_then(|sig| {
            let (sig, _) = sig;
            log::info!("got signal {:?}, stopping", sig);
            stopper.send(()).expect("send stop event");
            Ok(())
        })
        .map_err(|e| {
            let (e, _) = e;
            log::error!("error catching signal: {}", e);
        })
}
