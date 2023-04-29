use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct AuthenticationRequest {
    #[serde(rename = "User")]
    pub user: User,
    #[serde(rename = "Secret")]
    pub secret: Secret,
}

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    #[serde(rename = "isAdmin")]
    pub is_admin: bool,
}

#[derive(Default, PartialEq, Eq, Debug, Deserialize)]
pub struct Secret {
    pub password: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthenticationToken(String);
