extern crate hyper;
extern crate log;
extern crate futures;
extern crate signal;

use hyper::rt::Future;
use futures::sync::oneshot::{Sender, Receiver, channel};
use signal::trap::Trap;
use signal::Signal::{SIGINT, SIGTERM};

mod handler;
mod threadpool;


pub struct App {
    address: [u8; 4],
    port: u16,
    thread: threadpool::ThreadPool,
    terminator: Option<Sender<()>>,
}

impl App {
    pub fn new(port: u16, local_only: bool) -> App {
        let address = {
            if local_only {
                [127, 0, 0, 1]
            } else {
                [0, 0, 0, 0]
            }
        };

        App {
            address: address,
            port: port,
            thread: threadpool::ThreadPool::new(1),
            terminator: None,
        }
    }

    pub fn start(&mut self) {
        if self.terminator.is_some() {
            panic!("already started");
        }
        let (tx, rx) = channel::<()>();

        self.terminator = Some(tx);

        let a = self.address;
        let p = self.port;
        self.thread.execute(move || {
            run_server(rx, a, p);
        }).expect("failed to execute");
    }

    pub fn wait(&mut self) {
        log::info!("send SIGINT or SIGTERM to stop");

        let sigs = Trap::trap(&[SIGINT, SIGTERM]);
        for sig in sigs {
            log::info!("got signal {:?}, stopping", sig);
            self.stop();
            break;
        }
    }


    pub fn stop(&mut self) {
        if let Some(tx) = self.terminator.take() {
            let _ = tx.send(());
        }

        assert!(self.terminator.is_none());
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run_server(terminator: Receiver<()>, address: [u8; 4], port: u16) {
    log::debug!("starting server");

    let address = (address, port).into();
    let server = hyper::Server::bind(&address)
        .serve(|| hyper::service::service_fn_ok(handler::handle));

    log::info!("listening on {:?}", address);

    let server = server
        .with_graceful_shutdown(terminator)
        .map_err(|e| { log::error!("server error {}", e)});

    hyper::rt::run(server);
}
