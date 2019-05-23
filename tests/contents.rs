mod common;

use common::*;


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
