#[cfg(test)]
mod tests;

use crate::database;

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
#[serde(untagged)]
pub enum PackageData {
    Content {
        #[serde(rename = "Content")]
        content: String,
    },

    Url {
        #[serde(rename = "URL")]
        url: String,
    },
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Package {
    pub metadata: PackageMetadata,
    pub data: PackageData,
}

impl From<database::DatabaseEntry> for Package {
    fn from(database::DatabaseEntry { metadata, url, .. }: database::DatabaseEntry) -> Self {
        Package {
            metadata,
            data: PackageData::Url { url },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PackageWithUrl {
    #[serde(flatten)]
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
    #[serde(rename = "PullRequest")]
    pub pull_request: f64,
    #[serde(rename = "NetScore")]
    pub net_score: f64,
}

impl PackageRating {
    pub fn set_net_score(self) -> Self {
        PackageRating {
            net_score: (self.bus_factor
                + self.correctness
                + self.ramp_up
                + self.responsive_maintainer
                + self.license_score
                + self.good_pinning_practice
                + self.pull_request)
                / 7.,
            ..self
        }
    }
}

pub const RATING_FIELDS: [&str; 8] = [
    database::NET_SCORE,
    database::BUS_FACTOR,
    database::CORRECTNESS,
    database::RAMP_UP,
    database::RESPONSIVE_MAINTAINER,
    database::LICENSE_SCORE,
    database::GOOD_PINNING_PRACTICE,
    database::PULL_REQUEST,
];
