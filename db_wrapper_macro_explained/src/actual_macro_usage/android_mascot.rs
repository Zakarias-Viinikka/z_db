use crate::{create_the_entire_universe, payload::*};
use std::sync::{Mutex, MutexGuard};

macro_rules! unwrap_or_bail { ($result:expr) => { $result? }; }
macro_rules! decode_input { ($data:expr) => { $data }; }
macro_rules! finish_output { ($value:expr) => { Ok($value) }; }
macro_rules! method_input_type { ($typed:ty) => { $typed }; }
macro_rules! method_return_type { ($typed:ty) => { Result<$typed, DummyError> }; }

#[derive(uniffi::Object)]
pub struct AndroidMascot {
    conn: Mutex<String>,
}

#[uniffi::export]
impl AndroidMascot {
    #[uniffi::constructor]
    pub fn new(conn: String) -> Self {
        Self { conn: Mutex::new(conn) }
    }
}

impl AndroidMascot {
    fn get_conn(&self) -> Result<MutexGuard<'_, String>, DummyError> {
        self.conn
            .lock()
            .map_err(|e| DummyError::Msg(format!("lock failed: {e}")))
    }
}

create_the_entire_universe!(AndroidMascot, uniffi::export);
