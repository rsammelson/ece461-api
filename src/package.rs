use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::user::User;

#[derive(Default, Deserialize, Serialize)]
pub struct PackageMetadata {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "ID")]
    pub id: PackageId,
}

#[derive(Default, Serialize, Deserialize)]
pub struct PackageId(Uuid);

impl PackageId {
    pub fn new() -> Self {
        PackageId(Uuid::new_v4())
    }
}

impl ToString for PackageId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct PackageData {
    #[serde(rename = "Content")]
    pub content: String,
    #[serde(rename = "URL")]
    pub url: String,
    #[serde(rename = "JSProgram")]
    pub js_program: String,
}

#[derive(Default, Deserialize, Serialize)]
pub struct Package {
    pub metadata: PackageMetadata,
    pub data: PackageData,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Version")]
    pub version: Option<String>,
}

#[derive(Default, Serialize)]
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

#[derive(Default, Serialize)]
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

#[derive(Default, Serialize)]
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
