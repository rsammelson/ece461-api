use std::ops::Deref;

#[derive(Debug, PartialEq)]
pub struct DateTime(chrono::DateTime<chrono::Utc>);

impl DateTime {
    pub fn now() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

impl Deref for DateTime {
    type Target = chrono::DateTime<chrono::Utc>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        let dt = chrono::DateTime::parse_from_rfc3339(s).map_err(serde::de::Error::custom)?;
        Ok(DateTime(dt.into()))
    }
}

#[cfg(test)]
mod tests {
    use chrono::offset::TimeZone;
    use chrono::prelude::Utc;
    use serde_test::{assert_de_tokens, Token};

    #[test]
    fn datetime_test() {
        // 2023-02-20T06:36:32-10:30
        let answer = super::DateTime(Utc.with_ymd_and_hms(2023, 02, 20, 17, 23, 32).unwrap());
        assert_de_tokens(&answer, &[Token::BorrowedStr("2023-02-20T06:36:32-10:47")]);
    }
}
