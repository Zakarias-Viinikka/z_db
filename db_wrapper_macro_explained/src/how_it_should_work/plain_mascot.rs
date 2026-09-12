use crate::payload::*;

pub struct PlainMascot {
    conn: String,
}

impl PlainMascot {
    pub fn get_thing(&self, data: GetThingIn) -> Result<GetThingOut, DummyError> {
        let value = crate::db_operations::get_thing(&self.conn, data.id)?;
        Ok(GetThingOut { value })
    }

    pub fn get_combined(&self, data: GetCombinedIn) -> Result<CombinedOut, DummyError> {
        let value = crate::db_operations::get_thing(&self.conn, data.id)?;
        let other = crate::db_operations::get_other_thing(&self.conn)?;
        Ok(CombinedOut { value, other })
    }
}
