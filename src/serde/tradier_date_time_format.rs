use chrono::{DateTime, Local, Utc};
use serde::{self, Deserialize, Deserializer};

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|d| d.parse::<i64>().map_err(serde::de::Error::custom))
        .and_then(|millis| {
            DateTime::<Utc>::from_timestamp_millis(millis)
                .ok_or_else(|| serde::de::Error::custom("Invalid timestamp."))
        })
        .map(|utc| utc.with_timezone(&Local))
}
