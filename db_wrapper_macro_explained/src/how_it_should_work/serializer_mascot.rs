use crate::payload::*;
use crate::serialization::Convert;

macro_rules! unwrap_or_bail {
    ($result:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => return e.to_payload(),
        }
    };
}

// mirrors mascot.rs: direct connection, byte payload
pub struct SerializerMascot {
    conn: String,
}

impl SerializerMascot {
    pub fn get_thing(&self, data: Vec<u8>) -> Vec<u8> {
        let data = unwrap_or_bail!(GetThingIn::un_payloadify(&data));
        let value = unwrap_or_bail!(db_get_thing(&self.conn, data.id));
        GetThingOut { value }.to_payload()
    }
}
