use chrono::NaiveDate;
use serde::{self, Deserialize, Deserializer};

pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&date.format("%Y-%m-%d").to_string())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom))
}
