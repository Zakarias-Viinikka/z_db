use crate::{create_the_entire_universe, payload::*};
use crate::serialization::Convert;

macro_rules! unwrap_or_bail {
    ($result:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => return e.to_payload(),
        }
    };
}
macro_rules! decode_input { ($data:expr) => { unwrap_or_bail!(<_>::un_payloadify(&$data)) }; }
macro_rules! finish_output { ($value:expr) => { $value.to_payload() }; }
macro_rules! method_input_type { ($typed:ty) => { Vec<u8> }; }
macro_rules! method_return_type { ($typed:ty) => { Vec<u8> }; }

pub struct SerializerMascot {
    pub conn: String,
}

impl SerializerMascot {
    fn get_conn(&self) -> Result<&String, DummyError> {
        Ok(&self.conn)
    }
}

create_the_entire_universe!(SerializerMascot);
