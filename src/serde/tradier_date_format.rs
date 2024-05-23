use chrono::NaiveDate;
use serde::{self, Deserialize, Deserializer};

pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom))
}
