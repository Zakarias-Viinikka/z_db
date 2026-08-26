use crate::messages::{Message, Response};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::error::DbError;

pub trait Convert: Serialize + DeserializeOwned {
    fn to_payload(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_else(|e| {
            bincode::serialize(&DbError::SerializeError(
                "Failed to serialize: ".to_string() + &e.to_string(),
            ))
            .unwrap()
        })
    }

    fn un_payloadify(data: &[u8]) -> Result<Self, DbError> {
        bincode::deserialize(data).map_err(|e| DbError::CureFail(e.to_string()))
    }
}

pub fn ok_serialized() -> Vec<u8> {
    Ok::<(), DbError>(()).to_payload()
}

impl<T: Serialize + DeserializeOwned> Convert for T {}

// ---
// Base64 stuff below and vecu8 above
// ---

/// Wrapper around Vec<u8> that serializes as a base64 string in JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Base64Bytes(pub Vec<u8>);
impl Serialize for Base64Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&self.0);
        serializer.serialize_str(&encoded)
    }
}
impl<'de> Deserialize<'de> for Base64Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use base64::Engine;
        let s = String::deserialize(deserializer)?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(s)
            .map_err(serde::de::Error::custom)?;
        Ok(Base64Bytes(bytes))
    }
}

pub fn message_to_json_str(msg: &Message) -> Result<String, serde_json::Error> {
    serde_json::to_string(msg)
}
pub fn json_str_to_message(json: &str) -> Result<Message, serde_json::Error> {
    serde_json::from_str(json)
}
pub fn response_to_json_str(resp: &Response) -> Result<String, serde_json::Error> {
    serde_json::to_string(resp)
}
pub fn json_str_to_response(json: &str) -> Result<Response, serde_json::Error> {
    serde_json::from_str(json)
}

/*pub fn i_dont_want_to() -> Vec<u8> {
    DbError::BadCode("server does not handle this request".to_string()).serialize_wrapper()
}*/
