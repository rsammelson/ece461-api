use super::*;
use uuid::Uuid;

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
            body: "\"abc\"".to_string()
        }
    );
}

#[tokio::test]
#[ignore]
async fn query_search() {
    let query = vec![SearchQuery {
        name: "to_search".to_string(),
        version: None,
    }];
    let MyResponse {
        code,
        headers,
        body,
    } = search_packages(Json(query)).await.unwrap();

    let body: Vec<PackageMetadata> = serde_json::from_str(&body).unwrap();

    assert_eq!(code, StatusCode::OK);
    assert_eq!(
        headers,
        vec![(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json")
        )]
    );

    assert_eq!(
        body,
        vec![
            PackageMetadata {
                name: "to_search".to_string(),
                version: "1.0.0".to_string(),
                id: Uuid::nil().into()
            },
            PackageMetadata {
                name: "to_search".to_string(),
                version: "1.0.1".to_string(),
                id: Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8")
                    .unwrap()
                    .into()
            }
        ]
    )
}