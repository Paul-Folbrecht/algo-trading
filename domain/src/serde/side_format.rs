use crate::domain::Side;
use serde::{self, Deserialize, Deserializer};

pub fn deserialize<'de, D>(deserializer: D) -> Result<Side, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).and_then(|s| {
        if s == "buy" {
            Ok(Side::Buy)
        } else {
            Ok(Side::Sell)
        }
    })
}
