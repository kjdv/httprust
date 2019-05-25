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

fn sha256(s: String) -> String {
    use sha2::{Sha256, Digest};

    let mut hasher = Sha256::new();
    hasher.input(s.as_bytes());
    format!("{:x}", hasher.result())
}

#[test]
fn content_is_intact() {
    server();

    let mut response = Client::builder()
        .gzip(false)
        .build()
        .unwrap()
        .get(make_uri("large.txt").as_str())
        .send()
        .expect("request failed");

    assert_eq!(StatusCode::OK, response.status());
    assert_eq!(
        "223bdc7cb024ebe16c5fb1f10c47812eabbd51039cc7c67b10729501e4bdb577",
        sha256(response.text().unwrap())
    );
}

#[test]
fn content_is_intact_when_compressed() {
    server();

    let mut response = Client::builder()
        .gzip(true)
        .build()
        .unwrap()
        .get(make_uri("large.txt").as_str())
        .send()
        .expect("request failed");

    assert_eq!(StatusCode::OK, response.status());
    assert_eq!(
        "223bdc7cb024ebe16c5fb1f10c47812eabbd51039cc7c67b10729501e4bdb577",
        sha256(response.text().unwrap())
    );
}
