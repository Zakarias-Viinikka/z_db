use serde::{Serialize, de::DeserializeOwned};
use crate::payload::DummyError;

pub trait Convert: Serialize + DeserializeOwned {
    fn to_payload(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    fn un_payloadify(data: &[u8]) -> Result<Self, DummyError> {
        bincode::deserialize(data).map_err(|e| DummyError(e.to_string()))
    }
}

impl<T: Serialize + DeserializeOwned> Convert for T {}
