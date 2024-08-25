use crate::domain::Side;
use serde::{self, Deserialize, Deserializer};

pub fn serialize<S>(side: &Side, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match side {
        Side::Buy => serializer.serialize_str("buy"),
        Side::Sell => serializer.serialize_str("sell"),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Side, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).map(|s| if s == "buy" { Side::Buy } else { Side::Sell })
}
