use axum::{
    body::{Bytes, HttpBody},
    middleware::Next,
    response::{IntoResponse, Response},
};
use http::request::Parts;
use hyper::{Body, Request, StatusCode};
use std::fmt::Display;

enum Direction<'a> {
    Request(&'a Parts),
    Response,
}

impl Display for Direction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Request(parts) => write!(f, "request {} {}", parts.method, parts.uri),
            Direction::Response => f.write_str("response"),
        }
    }
}

// https://github.com/tokio-rs/axum/blob/main/examples/print-request-response/src/main.rs
pub async fn print_request_response<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, String)>
where
    B: HttpBody<Data = Bytes>,
    B: From<Bytes>,
    B::Error: std::fmt::Display,
{
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print(Direction::Request(&parts), body).await?;
    let req = Request::from_parts(parts, B::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print(Direction::Response, body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(
    direction: Direction<'_>,
    body: B,
) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match hyper::body::to_bytes(body).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {} body: {}", direction, err),
            ));
        }
    };

    log::info!("{} body = `{}`", direction, String::from_utf8_lossy(&bytes));

    Ok(bytes)
}
