use crate::queries::types::{PackageMetadata, PackageRating};

use firestore::FirestoreDb;
use serde::{Deserialize, Serialize};

#[cfg(not(test))]
pub const METADATA: &'static str = "metadata";
#[cfg(test)]
pub const METADATA: &'static str = "metadata-test";

#[cfg(not(test))]
pub const PAGE_LIMIT: usize = 10;
#[cfg(test)]
pub const PAGE_LIMIT: usize = 2;

pub async fn get_database() -> FirestoreDb {
    FirestoreDb::new("ece-461-dev").await.unwrap()
}

#[derive(Deserialize, Serialize)]
pub struct DatabaseEntry {
    #[serde(flatten)]
    pub metadata: PackageMetadata,
    #[serde(rename = "URL")]
    pub url: String,
    #[serde(flatten)]
    pub rating: PackageRating,
}

pub const NAME: &str = "Name";
pub const VERSION: &str = "Version";
pub const ID: &str = "ID";
pub const URL: &str = "URL";

pub const BUS_FACTOR: &str = "BusFactor";
pub const CORRECTNESS: &str = "Correctness";
pub const RAMP_UP: &str = "RampUp";
pub const RESPONSIVE_MAINTAINER: &str = "ResponsiveMaintainer";
pub const LICENSE_SCORE: &str = "LicenseScore";
pub const GOOD_PINNING_PRACTICE: &str = "GoodPinningPractice";
