use crate::payload::DummyError;

// Mock of the real db crate's API surface (black_magic-style fns).
// Mascots call into this; the macro wraps conn access + serialization around it.

pub fn get_thing(conn: &str, id: u32) -> Result<String, DummyError> {
    Ok(format!("{conn} says thing #{id}"))
}

pub fn get_other_thing(conn: &str) -> Result<u32, DummyError> {
    Ok(conn.len() as u32)
}
