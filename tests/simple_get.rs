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
