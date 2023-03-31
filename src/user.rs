use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AuthenticationRequest {
    #[serde(rename = "User")]
    pub user: User,
    #[serde(rename = "Secret")]
    pub secret: Secret,
}

#[derive(Default, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "isAdmin")]
    pub is_admin: bool,
}

#[derive(Default, Deserialize)]
pub struct Secret {
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthenticationToken(String);

impl AuthenticationToken {
    pub fn new(auth: AuthenticationRequest) -> Self {
        AuthenticationToken({
            let mut ret = auth.user.name;
            ret.push_str(&auth.secret.password);
            ret
        })
    }
}
