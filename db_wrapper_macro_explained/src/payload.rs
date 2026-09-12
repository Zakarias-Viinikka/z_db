use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DummyError(pub String);

#[derive(Serialize, Deserialize)]
pub struct GetThingIn {
    pub id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct GetThingOut {
    pub value: String,
}

// stand-in for something like black_magic::table_shape
pub fn db_get_thing(conn: &str, id: u32) -> Result<String, DummyError> {
    Ok(format!("{conn} says thing #{id}"))
}
