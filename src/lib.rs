extern crate hyper;
extern crate log;
extern crate futures;
extern crate tokio;
extern crate tokio_signal;

use futures::Stream;
use hyper::rt::Future;
use futures::sync::oneshot::{Sender, channel};

use tokio::runtime::current_thread;
use tokio_signal::unix::{Signal, SIGINT, SIGTERM};

mod handler;

pub struct Config {
    pub port: u16,
    pub local_only: bool,
}

pub fn run(cfg: Config) {
    log::debug!("starting server");

    let sigint = Signal::new(SIGINT).flatten_stream();
    let sigterm = Signal::new(SIGTERM).flatten_stream();
    let stream = sigint.select(sigterm);

    let mut runtime = current_thread::Runtime::new().expect("new runtime");
    let (server, stopper) = make_server(cfg);

    runtime.spawn(server);

    let (signal, _) = runtime
        .block_on(stream.into_future())
        .map_err(|_| {
            log::error!("fail waiting for signal");
        }).expect("blocking on signals");

    log::info!("received signal {:?}, stopping", signal);
    stopper.send(()).expect("sending stop");
    runtime.run().expect("final run");
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

    let server = hyper::Server::bind(&address)
        .serve(|| hyper::service::service_fn_ok(handler::handle));

    log::info!("listening on {:?}", address);

    let server = server
        .with_graceful_shutdown(rx)
        .map_err(|e| { log::error!("server error {}", e)});

    (server, tx)
}
