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

pub struct SerializerMascot {
    conn: String,
}

impl SerializerMascot {
    pub fn get_thing(&self, data: Vec<u8>) -> Vec<u8> {
        let data = unwrap_or_bail!(GetThingIn::un_payloadify(&data));
        let value = unwrap_or_bail!(crate::db_operations::get_thing(&self.conn, data.id));
        GetThingOut { value }.to_payload()
    }

    pub fn get_combined(&self, data: Vec<u8>) -> Vec<u8> {
        let data = unwrap_or_bail!(GetCombinedIn::un_payloadify(&data));
        let value = unwrap_or_bail!(crate::db_operations::get_thing(&self.conn, data.id));
        let other = unwrap_or_bail!(crate::db_operations::get_other_thing(&self.conn));
        CombinedOut { value, other }.to_payload()
    }
}
