#[cfg(test)]
mod tests;

use crate::{
    database::{self, DatabaseEntry},
    user::User,
};

use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PackageMetadata {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Version")]
    pub version: Version,
    #[serde(rename = "ID")]
    pub id: PackageId,
}

pub const PACKAGE_METADATA_FIELDS: [&str; 3] = [database::NAME, database::VERSION, database::ID];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageId(String);

impl PackageId {
    pub fn new() -> Self {
        Uuid::new_v4().into()
    }
}

impl<T> From<T> for PackageId
where
    T: ToString,
{
    fn from(value: T) -> Self {
        PackageId(value.to_string())
    }
}

impl AsRef<str> for PackageId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for PackageId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum PackageData {
    #[serde(rename = "Content")]
    Content(String),
    #[serde(rename = "URL")]
    Url(String),
    #[serde(rename = "JSProgram")]
    JsProgram(String),
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Package {
    pub metadata: PackageMetadata,
    pub data: PackageData,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PackageWithUrl {
    pub metadata: PackageMetadata,
    #[serde(rename = "URL")]
    pub url: String,
}

pub const PACKAGE_FIELDS: [&str; 4] = [
    database::NAME,
    database::VERSION,
    database::ID,
    database::URL,
];

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct PackageHistoryEntry {
    #[serde(rename = "User")]
    pub user: User,
    #[serde(rename = "Date")]
    pub date: DateTime<Utc>,
    #[serde(rename = "PackageMetadata")]
    pub metadata: PackageMetadata,
    #[serde(rename = "Action")]
    pub action: PackageHistoryAction,
}

#[cfg_attr(test, derive(strum::EnumIter))]
#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum PackageHistoryAction {
    #[serde(rename = "CREATE")]
    Create,
    #[serde(rename = "UPDATE")]
    Update,
    #[serde(rename = "DOWNLOAD")]
    Download,
    #[serde(rename = "RATE")]
    Rate,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageRating {
    #[serde(rename = "BusFactor")]
    pub bus_factor: f64,
    #[serde(rename = "Correctness")]
    pub correctness: f64,
    #[serde(rename = "RampUp")]
    pub ramp_up: f64,
    #[serde(rename = "ResponsiveMaintainer")]
    pub responsive_maintainer: f64,
    #[serde(rename = "LicenseScore")]
    pub license_score: f64,
    #[serde(rename = "GoodPinningPractice")]
    pub good_pinning_practice: f64,
}

pub const RATING_FIELDS: [&str; 6] = [
    database::BUS_FACTOR,
    database::CORRECTNESS,
    database::RAMP_UP,
    database::RESPONSIVE_MAINTAINER,
    database::LICENSE_SCORE,
    database::GOOD_PINNING_PRACTICE,
];
