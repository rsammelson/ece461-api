use super::*;

use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[test]
fn des_ser_metadata() {
    let data =
        r#"{"Name":"test_package","Version":"1.2.3","ID":"00000000-0000-0000-0000-000000000000"}"#;

    let deserialized: PackageMetadata = serde_json::from_str(data).unwrap();
    assert_eq!(
        deserialized,
        PackageMetadata {
            name: "test_package".to_string(),
            version: "1.2.3".to_string(),
            id: Uuid::nil().into()
        }
    );

    let serialized = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(serialized, data);
}

#[test]
fn metadata_all_required() {
    let data = r#"{"Name":"package_test","Version":"3.2.1"}"#;
    let deserialized: Result<PackageMetadata, _> = serde_json::from_str(data);
    if let Ok(_) = deserialized {
        panic!("Expected \"ID\" to be a required field");
    }

    let data = r#"{"Name":"package_test","ID":"00000000-0000-0000-0000-000000000000"}"#;
    let deserialized: Result<PackageMetadata, _> = serde_json::from_str(data);
    if let Ok(_) = deserialized {
        panic!("Expected \"Version\" to be a required field");
    }

    let data = r#"{"Version":"0.0.1","ID":"00000000-0000-0000-0000-000000000000"}"#;
    let deserialized: Result<PackageMetadata, _> = serde_json::from_str(data);
    if let Ok(_) = deserialized {
        panic!("Expected \"Name\" to be a required field");
    }
}

#[test]
fn des_ser_data_content() {
    let data = r#"{"Content":"abc"}"#;

    let deserialized: PackageData = serde_json::from_str(data).unwrap();
    assert_eq!(deserialized, PackageData::Content("abc".to_string()));

    let serialized = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(serialized, data);
}

#[test]
fn des_ser_data_url() {
    let data = r#"{"URL":"https://example.com"}"#;

    let deserialized: PackageData = serde_json::from_str(data).unwrap();
    assert_eq!(
        deserialized,
        PackageData::Url("https://example.com".to_string())
    );

    let serialized = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(serialized, data);
}

#[test]
fn des_ser_data_js() {
    let data = r#"{"JSProgram":"return 1 + 2;"}"#;

    let deserialized: PackageData = serde_json::from_str(data).unwrap();
    assert_eq!(
        deserialized,
        PackageData::JsProgram("return 1 + 2;".to_string())
    );

    let serialized = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(serialized, data);
}

#[test]
fn data_only_one_field() {
    let data = r#"{"Content":"abc", "JSProgram":"return 1 + 2;"}"#;
    let deserialized: Result<PackageData, _> = serde_json::from_str(data);
    if let Ok(_) = deserialized {
        panic!("Expected to only be able to set one of the fields of `data`");
    }
}

#[test]
fn des_search() {
    let data = r#"{"Name":"to_search","Version":"1.0"}"#;

    let deserialized: SearchQuery = serde_json::from_str(data).unwrap();
    assert_eq!(
        deserialized,
        SearchQuery {
            name: "to_search".to_string(),
            version: Some("1.0".to_string())
        }
    );
}

#[test]
fn des_search_no_version() {
    let data = r#"{"Name":"to_search"}"#;

    let deserialized: SearchQuery = serde_json::from_str(data).unwrap();
    assert_eq!(
        deserialized,
        SearchQuery {
            name: "to_search".to_string(),
            version: None,
        }
    );
}

#[test]
fn des_search_name_required() {
    let data = r#"{"Version":"0.0.0"}"#;

    let deserialized: Result<SearchQuery, _> = serde_json::from_str(data);
    if let Ok(_) = deserialized {
        panic!("Expected \"Name\" to be required for a `SearchQuery`");
    }
}

#[test]
fn ser_history_entry() {
    let naivedatetime_utc = NaiveDate::from_ymd_opt(2000, 1, 12)
        .unwrap()
        .and_hms_opt(2, 0, 0)
        .unwrap();

    let data = PackageHistoryEntry {
        user: User {
            name: "jim".to_string(),
            is_admin: false,
        },
        date: DateTime::from_utc(naivedatetime_utc, Utc),
        metadata: Default::default(),
        action: PackageHistoryAction::Create,
    };

    let serialized = serde_json::to_string(&data).unwrap();
    assert_eq!(
        serialized,
        r#"{"User":{"Name":"jim","isAdmin":false},"Date":"2000-01-12T02:00:00Z","PackageMetadata":{"Name":"","Version":"","ID":"00000000-0000-0000-0000-000000000000"},"Action":"CREATE"}"#
    )
}

#[test]
fn ser_history_actions() {
    use strum::IntoEnumIterator;
    let data = PackageHistoryAction::iter().collect::<Vec<_>>();

    let serialized = serde_json::to_string(&data).unwrap();
    assert_eq!(serialized, r#"["CREATE","UPDATE","DOWNLOAD","RATE"]"#);
}
