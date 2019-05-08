mod common;

use common::*;

#[test]
fn simple_get() {
    server();

    get("index.html", is_ok);
}

#[test]
fn simple_get2() {
    server();

    get("index.html", is_ok);
}

#[test]
fn simple_get3() {
    server();

    get("index.html", is_ok);
}
