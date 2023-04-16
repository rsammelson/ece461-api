mod datetime;
pub(super) mod graphql;

use super::{
    url::{get_client, GithubUrl},
    ScoringData,
};

use once_cell::sync::OnceCell;

#[derive(Debug, thiserror::Error)]
pub enum GraphQlError {
    #[error("No data in response")]
    MissingData,
    #[error("{0}")]
    ReqwestError(#[from] reqwest::Error),
}

fn get_token() -> &'static str {
    static TOKEN: OnceCell<String> = OnceCell::new();
    TOKEN.get_or_init(|| std::env::var("GITHUB_TOKEN").unwrap())
}
