extern crate hyper_rustls;
extern crate rustls;
extern crate tokio_rustls;
use super::TlsConfig;
use futures::Stream;
use hyper;
use log;
use std::io::Result;
use std::sync::Arc;
use tokio;

// most code below kindly taken from hyper-rustls example code

pub fn configure_tls(cfg: TlsConfig) -> Result<Arc<rustls::ServerConfig>> {
    log::debug!("configuring tls");

    // Build TLS configuration.

    // Load public certificate.
    let certs = load_certs(cfg.certificate_file.as_str())?;
    // Load private key.
    let key = load_private_key(cfg.private_key_file.as_str())?;
    // Do not use client certificate authentication.
    let mut server_cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
    server_cfg.alpn_protocols = vec![b"h2".to_vec(), b"http1.1".to_vec()];

    // Select a certificate to use.
    server_cfg
        .set_single_cert(certs, key)
        .map_err(|e| error(format!("{}", e)))?;
    Ok(Arc::new(server_cfg))
}

pub fn make_server(
    address: std::net::SocketAddr,
    cfg: Arc<rustls::ServerConfig>,
) -> Result<
    hyper::server::Builder<
        impl Stream<
            Item = tokio_rustls::TlsStream<tokio::net::TcpStream, rustls::ServerSession>,
            Error = std::io::Error,
        >,
    >,
> {
    log::debug!("serving tls at {}", address);

    // Create a TCP listener via tokio.
    let tcp = tokio::net::tcp::TcpListener::bind(&address)?;
    let tls_acceptor = tokio_rustls::TlsAcceptor::from(cfg);
    // Prepare a long-running future stream to accept and serve cients.
    let tls = tcp
        .incoming()
        .and_then(move |s| tls_acceptor.accept(s))
        .then(|r| match r {
            Ok(x) => Ok::<_, std::io::Error>(Some(x)),
            Err(_e) => {
                log::error!("{}", _e);
                Ok(None)
            }
        })
        .filter_map(|x| x);

    let builder = hyper::Server::builder(tls);
    Ok(builder)
}

fn error(err: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, err)
}

// Load public certificate from file.
fn load_certs(filename: &str) -> Result<Vec<rustls::Certificate>> {
    // Open certificate file.
    let certfile = std::fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = std::io::BufReader::new(certfile);

    // Load and return certificate.
    rustls::internal::pemfile::certs(&mut reader)
        .map_err(|_| error("failed to load certificate".into()))
}

// Load private key from file.
fn load_private_key(filename: &str) -> Result<rustls::PrivateKey> {
    // Open keyfile.
    let keyfile = std::fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = std::io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = rustls::internal::pemfile::rsa_private_keys(&mut reader)
        .map_err(|_| error("failed to load private key".into()))?;
    if keys.len() != 1 {
        return Err(error("expected a single private key".into()));
    }
    Ok(keys[0].clone())
}
