use chrono::{DateTime, FixedOffset, Local};
use serde::{self, Deserialize, Deserializer};

pub fn serialize<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_i64(date.timestamp_millis())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
where
    D: Deserializer<'de>,
{
    // RFC3339 format: 2024-06-17T13:45:27.304Z
    String::deserialize(deserializer)
        .and_then(|s| {
            DateTime::<FixedOffset>::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)
        })
        .map(|utc| utc.with_timezone(&Local))
}
