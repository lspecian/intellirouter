//! Utility functions for chain definitions
//!
//! This file contains utility functions and helpers for chain definitions.

/// Module for serializing/deserializing Duration
pub mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(duration) => serializer.serialize_u64(duration.as_secs()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = Option::<u64>::deserialize(deserializer)?;
        Ok(seconds.map(Duration::from_secs))
    }
}
