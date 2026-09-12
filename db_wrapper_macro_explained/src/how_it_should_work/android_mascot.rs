use crate::payload::*;
use std::sync::{Mutex, MutexGuard};

// Mirrors android_mascot.rs but WITHOUT uniffi attrs, so it doesn't collide
// with the real exported one. This is a shape reference, not an export.

pub struct AndroidMascot {
    conn: Mutex<String>,
}

impl AndroidMascot {
    pub fn new(conn: String) -> Self {
        Self { conn: Mutex::new(conn) }
    }

    pub fn get_thing(&self, data: GetThingIn) -> Result<GetThingOut, DummyError> {
        let conn = self.get_conn()?;
        let value = crate::db_operations::get_thing(&conn, data.id)?;
        Ok(GetThingOut { value })
    }

    pub fn get_combined(&self, data: GetCombinedIn) -> Result<CombinedOut, DummyError> {
        let conn = self.get_conn()?;
        let value = crate::db_operations::get_thing(&conn, data.id)?;
        let other = crate::db_operations::get_other_thing(&conn)?;
        Ok(CombinedOut { value, other })
    }

    fn get_conn(&self) -> Result<MutexGuard<'_, String>, DummyError> {
        self.conn
            .lock()
            .map_err(|e| DummyError::Msg(format!("lock failed: {e}")))
    }
}
