use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, uniffi::Error)]
pub enum DummyError {
    Msg(String),
}

impl std::fmt::Display for DummyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DummyError::Msg(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Serialize, Deserialize, uniffi::Record)]
pub struct GetThingIn {
    pub id: u32,
}

#[derive(Serialize, Deserialize, uniffi::Record)]
pub struct GetThingOut {
    pub value: String,
}

#[derive(Serialize, Deserialize, uniffi::Record)]
pub struct GetCombinedIn {
    pub id: u32,
}

#[derive(Serialize, Deserialize, uniffi::Record)]
pub struct CombinedOut {
    pub value: String,
    pub other: u32,
}
