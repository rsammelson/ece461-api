#[cfg(test)]
mod tests;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::user::User;

#[derive(Default, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PackageMetadata {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "ID")]
    pub id: PackageId,
}

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageId(Uuid);

impl PackageId {
    pub fn new() -> Self {
        PackageId(Uuid::new_v4())
    }
}

impl From<Uuid> for PackageId {
    fn from(value: Uuid) -> Self {
        PackageId(value)
    }
}

impl ToString for PackageId {
    fn to_string(&self) -> String {
        self.0.to_string()
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

impl Default for PackageData {
    fn default() -> Self {
        PackageData::Content(Default::default())
    }
}

#[derive(Default, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Package {
    pub metadata: PackageMetadata,
    pub data: PackageData,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct SearchQuery {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Version")]
    pub version: Option<String>,
}

#[derive(Default, Debug, PartialEq, Eq, Serialize)]
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
#[derive(Default, Debug, PartialEq, Eq, Serialize)]
pub enum PackageHistoryAction {
    #[default]
    #[serde(rename = "CREATE")]
    Create,
    #[serde(rename = "UPDATE")]
    Update,
    #[serde(rename = "DOWNLOAD")]
    Download,
    #[serde(rename = "RATE")]
    Rate,
}

#[derive(Default, Debug, Serialize)]
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