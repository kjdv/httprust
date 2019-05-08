extern crate hyper;
extern crate log;
extern crate futures;

use hyper::rt::Future;
use futures::sync::oneshot::{Sender, channel};

mod handler;


pub struct App {
    address: [u8; 4],
    port: u16,
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
            terminator: None,
        }
    }

    pub fn spawn(&mut self) -> Result<(), &'static str> {
        if self.terminator.is_some() {
            return Err("already spawned");
        }

        let address = (self.address, self.port).into();

        let server = hyper::Server::bind(&address)
            .serve(|| hyper::service::service_fn_ok(handler::handle));

        log::info!("listening on {:?}", address);

        let (tx, rx) = channel::<()>();

        let graceful = server
            .with_graceful_shutdown(rx)
            .map_err(|e| { log::error!("server error {}", e)});

        hyper::rt::spawn(graceful);

        self.terminator = Some(tx);

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.terminator.take() {
            tx.send(()).map_err(|_| {
                log::error!("error stopping server");
            }).unwrap();
        }

        assert!(self.terminator.is_none());
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.stop();
    }
}
