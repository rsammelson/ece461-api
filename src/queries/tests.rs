use super::*;

#[test]
fn basic_ok_response() {
    let resp = ok("abc");

    assert_eq!(
        resp,
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            "\"abc\"".to_string()
        )
    );
}
