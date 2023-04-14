use super::*;

#[test]
fn basic_ok_response() {
    let resp = ok("abc");

    assert_eq!(
        resp,
        MyResponse {
            code: StatusCode::OK,
            headers: vec![(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json")
            )],
            body: "abc"
        }
    );
}
