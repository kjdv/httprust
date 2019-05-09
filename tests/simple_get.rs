mod common;

use common::*;

#[test]
fn simple_get() {
    server();

    let response = get("index.html").expect("request failed");
    assert_eq!(StatusCode::OK, response.status());
}

#[test]
fn simple_get2() {
    server();

    let response = get("index.html").expect("request failed");
    assert_eq!(StatusCode::OK, response.status());
}

#[test]
fn simple_get3() {
    server();

    let response = get("index.html").expect("request failed");
    assert_eq!(StatusCode::OK, response.status());
}
