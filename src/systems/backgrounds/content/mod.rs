use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundTypes {
    Desert,
    Apocalypse,
}

impl BackgroundTypes {
    pub fn content(&self) -> &str {
        match self {
            BackgroundTypes::Desert => include_str!("./desert.json"),
            BackgroundTypes::Apocalypse => include_str!("./apocalypse.json"),
        }
    }
}
