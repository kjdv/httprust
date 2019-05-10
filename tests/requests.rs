mod common;

use common::*;

#[test]
fn simple_get() {
    server();

    let response = get("index.html").expect("request failed");
    assert_eq!(StatusCode::OK, response.status());
}

#[test]
fn http2_get() {
    server();

    let response = Client::builder()
        .h2_prior_knowledge()
        .build()
        .expect("build client")
        .get(make_uri("index.html").as_str())
        .send()
        .expect("fail send");

    assert_eq!(StatusCode::OK, response.status());
}

#[test]
fn get_content() {
    server();

    let mut response = get("hello.txt").expect("request failed");
    assert_eq!(StatusCode::OK, response.status());
    assert_eq!("hello!\n", response.text().unwrap());
}

#[test]
fn not_found() {
    server();

    let response = get("no_such_thing").unwrap();
    assert_eq!(StatusCode::NOT_FOUND, response.status());
}

#[test]
fn no_directory_traversal_allowed() {
    use std::io::{Read, Write};

    server();

    // doing this handcoded as reqwest already doesn't allow you to make such requests
    let mut stream = std::net::TcpStream::connect(format!("{}:{}", ADDRESS, PORT)).expect("connect");
    stream.write_all(b"GET /../requests.rs HTTP/1.1\r\n\r\n").expect("write");

    let mut buf = [0; 512];
    let r = stream.read(&mut buf).expect("read");
    let response = String::from(std::str::from_utf8(&buf[..r]).expect("utf-8"));
    assert!(response.starts_with("HTTP/1.1 403"));
}

#[test]
fn no_directory_traversal_allowed_for_non_existant() {
    // subtlety: if we have a traversal for a file that does not actually exist, we want this
    // reported as forbidden instead of not found: don't leak more information than you have to
    use std::io::{Read, Write};

    server();

    let mut stream = std::net::TcpStream::connect(format!("{}:{}", ADDRESS, PORT)).expect("connect");
    stream.write_all(b"GET /../no_such_thing HTTP/1.1\r\n\r\n").expect("write");

    let mut buf = [0; 512];
    let r = stream.read(&mut buf).expect("read");
    let response = String::from(std::str::from_utf8(&buf[..r]).expect("utf-8"));
    assert!(response.starts_with("HTTP/1.1 403"));
}

#[test]
fn no_post_on_static_file() {
    server();

    let response = post("index.html", b"blah").expect("post");
    assert_eq!(StatusCode::METHOD_NOT_ALLOWED, response.status());
}
