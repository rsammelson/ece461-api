use super::*;

use semver::Version;
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
            version: Version::parse("1.2.3").unwrap(),
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
