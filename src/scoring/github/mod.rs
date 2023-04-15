mod datetime;
pub(super) mod graphql;

use super::{url::GithubUrl, ScoringData};

use once_cell::sync::OnceCell;
use reqwest::Client;

#[derive(Debug, thiserror::Error)]
pub enum GraphQlError {
    #[error("No data in response")]
    MissingData,
    #[error("{0}")]
    ReqwestError(#[from] reqwest::Error),
}

pub fn get_client() -> Client {
    static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
    static CLIENT: OnceCell<Client> = OnceCell::new();
    CLIENT
        .get_or_init(|| {
            Client::builder()
                .user_agent(USER_AGENT)
                .https_only(true)
                .build()
                .unwrap()
        })
        .clone()
}
