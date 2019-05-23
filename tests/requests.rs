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
fn large_files_are_chunked() {
    server();

    let response = get("large.txt").expect("request failed");
    let transer_encoding = response.headers().get(reqwest::header::TRANSFER_ENCODING)
        .expect("expected transfer-encoding");

    assert_eq!("chunked", transer_encoding);
}

// we need to test for content serving as integration tests, as Tokio's async file io does not play
// nice with unit testing

#[test]
fn content_by_get() {
    server();

    let mut response = get("hello.txt").expect("request failed");

    assert_eq!(StatusCode::OK, response.status());
    assert_eq!("hello!\n", response.text().expect("some content"));
}

#[test]
fn head_returns_no_data() {
    server();

    let mut response = Client::new()
        .head(make_uri("hello.txt").as_str())
        .send()
        .expect("request failed");

    assert_eq!(StatusCode::OK, response.status());
    assert_eq!("", response.text().expect("some content"));
}
