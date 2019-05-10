mod common;

use common::*;

#[test]
fn not_found() {
    server();

    let response = get("no_such_thing").unwrap();
    assert_eq!(StatusCode::NOT_FOUND, response.status());
}
