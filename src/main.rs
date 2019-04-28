use std::io::prelude::*;
use std::fs;
use std::net::TcpStream;
use std::net::TcpListener;

mod threadpool;
use threadpool::ThreadPool;

mod handler;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                pool.execute(|| {
                    handle_connection(s);
                }).unwrap();
            }
            Err(error) => {
                eprintln!("error with incoming stream: {:?}", error);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    let contents = fs::read_to_string("samples/hello.html").unwrap();

    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", contents);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
