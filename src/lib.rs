extern crate hyper;
extern crate log;
extern crate futures;
extern crate tokio;
extern crate tokio_signal;

use std::sync::Arc;
use hyper::rt::{self, Future, Stream};
use futures::sync::oneshot::{Sender, channel};

use tokio_signal::unix::{Signal, SIGINT, SIGTERM};

mod handler;

pub struct Config {
    pub port: u16,
    pub local_only: bool,
    pub root: String,
}

pub fn run(cfg: Config) {
    log::debug!("starting server");

    rt::run(rt::lazy(move || {
        let (server, stopper) = make_server(cfg);
        let signal_handler = make_signal_handler(stopper);

        rt::spawn(signal_handler);
        rt::spawn(server);

        Ok(())
    }));

    log::info!("done");
}

fn make_server(cfg: Config) -> (impl Future<Item = (), Error = ()>, Sender<()>) {
    let address = {
        if cfg.local_only {
            [127, 0, 0, 1]
        } else {
            [0, 0, 0, 0]
        }
    };
    let address = (address, cfg.port).into();

    let (tx, rx) = channel::<()>();

    let handle = handler::Handler::new(cfg.root.as_str())
        .map_err(|e| {
            log::error!("error creating handler {}", e);
            panic!("create handler");
        }).unwrap();

    let handle = Arc::new(handle);

    let server = hyper::Server::bind(&address)
        .serve(move || {
            let this_handler = handle.clone();
            hyper::service::service_fn(move |req| {
                this_handler.handle(req)
            })
    });

    log::info!("listening on {:?}", address);

    let server = server
        .with_graceful_shutdown(rx)
        .map_err(|e| { log::error!("server error {}", e)});

    (server, tx)
}

fn make_signal_handler(stopper: Sender<()>) -> impl Future<Item = (), Error = ()> {
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
