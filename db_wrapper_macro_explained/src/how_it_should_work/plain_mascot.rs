use crate::payload::*;

// mirrors mascot.rs / testing_mascot.rs: direct connection, typed in/out
pub struct PlainMascot {
    conn: String,
}

impl PlainMascot {
    pub fn get_thing(&self, data: GetThingIn) -> Result<GetThingOut, DummyError> {
        let value = db_get_thing(&self.conn, data.id)?;
        Ok(GetThingOut { value })
    }
}
