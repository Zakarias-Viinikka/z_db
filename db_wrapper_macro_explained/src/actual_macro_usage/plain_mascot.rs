use crate::payload::*;

macro_rules! unwrap_or_bail {
    ($result:expr) => {
        $result?
    };
}

macro_rules! decode_input {
    ($data:expr) => {
        $data
    };
}

macro_rules! finish_output {
    ($value:expr) => {
        Ok($value)
    };
}

macro_rules! method_input_type {
    ($typed:ty) => {
        $typed
    };
}

macro_rules! method_return_type {
    ($typed:ty) => { Result<$typed, DummyError> };
}

macro_rules! method_attrs {
    () => {};
}

pub struct PlainMascot {
    conn: String,
}

impl PlainMascot {
    fn get_conn(&self) -> Result<&String, DummyError> {
        Ok(&self.conn)
    }

    crate::db_wrapper_method_generator!();
}
