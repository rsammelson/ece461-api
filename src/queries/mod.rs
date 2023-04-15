#[cfg(test)]
mod tests;

mod endpoints;
mod filter;
pub mod types;

pub use endpoints::*;

use axum::{
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::IntoResponse,
};
use serde::Serialize;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct MyResponse<T> {
    code: StatusCode,
    headers: Vec<(HeaderName, HeaderValue)>,
    body: T,
}

impl<T> MyResponse<T> {
    fn push_header(mut self, header: (HeaderName, HeaderValue)) -> Self {
        self.headers.push(header);
        self
    }
}

impl<T> IntoResponse for MyResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let headers = HeaderMap::from_iter(self.headers.into_iter());
        (
            self.code,
            headers,
            serde_json::to_string(&self.body).unwrap_or_default(),
        )
            .into_response()
    }
}

/// helper function for constructing common use case of returning status ok with json body
fn ok<T: Serialize>(body: T) -> MyResponse<T> {
    respond(StatusCode::OK, body)
}

fn respond<T: Serialize>(code: StatusCode, body: T) -> MyResponse<T> {
    MyResponse {
        code,
        headers: vec![(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )],
        body,
    }
}
