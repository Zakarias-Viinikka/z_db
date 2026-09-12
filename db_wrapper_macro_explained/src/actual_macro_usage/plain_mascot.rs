use crate::{create_the_entire_universe, payload::*};

macro_rules! unwrap_or_bail { ($result:expr) => { $result? }; }
macro_rules! decode_input { ($data:expr) => { $data }; }
macro_rules! finish_output { ($value:expr) => { Ok($value) }; }
macro_rules! method_input_type { ($typed:ty) => { $typed }; }
macro_rules! method_return_type { ($typed:ty) => { Result<$typed, DummyError> }; }

pub struct PlainMascot {
    pub conn: String,
}

impl PlainMascot {
    fn get_conn(&self) -> Result<&String, DummyError> {
        Ok(&self.conn)
    }
}

create_the_entire_universe!(PlainMascot);
