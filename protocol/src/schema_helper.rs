use std::marker::PhantomData;

use crate::row_col::Col;

//enum for columns
// method for destructing that takes the enum + col to destruct
// get_colum_name that the enum "points" to
// get_type cuz why not? might be useful

#[derive(Debug)]
pub enum TypeOfCol {
    Text,
    Integer,
    Real,
    Blob,
}

pub struct SchemaTable {
    pub table_name: &'static str,
}

pub struct SchemaColumn<T> {
    pub name: &'static str,
    pub type_of_col: &'static TypeOfCol,
    pub can_be_null: bool,
    pub _marker: PhantomData<T>,
}

const PANIC_EXPLANATION: &str = "this should only fail to unwrap if there's mistake in the schema or if a migration hasn't been done correctly afaik";
pub trait DestructDbReturnCol {
    type Output;
    fn try_destruct_db_col(&self, db_col: Col) -> Result<Option<Self::Output>, String>;
    fn destruct_db_col(&self, db_col: Col) -> Option<Self::Output>;
}

impl DestructDbReturnCol for SchemaColumn<i64> {
    type Output = i64;

    fn destruct_db_col(&self, db_col: Col) -> Option<Self::Output> {
        self.try_destruct_db_col(db_col).expect(PANIC_EXPLANATION)
    }

    fn try_destruct_db_col(&self, db_col: Col) -> Result<Option<Self::Output>, String> {
        match db_col {
            Col::Integer(value_as_i64) => Ok(Some(value_as_i64)),
            Col::Null(_) => Ok(None),
            _ => Err("illegal".into()),
        }
    }
}

impl DestructDbReturnCol for SchemaColumn<f64> {
    type Output = f64;

    fn destruct_db_col(&self, db_col: Col) -> Option<Self::Output> {
        self.try_destruct_db_col(db_col).expect(PANIC_EXPLANATION)
    }

    fn try_destruct_db_col(&self, db_col: Col) -> Result<Option<Self::Output>, String> {
        match db_col {
            Col::Real(value_as_f64) => Ok(Some(value_as_f64)),
            Col::Null(_) => Ok(None),
            _ => Err("illegal".into()),
        }
    }
}

impl DestructDbReturnCol for SchemaColumn<String> {
    type Output = String;

    fn destruct_db_col(&self, db_col: Col) -> Option<Self::Output> {
        self.try_destruct_db_col(db_col).expect(PANIC_EXPLANATION)
    }

    fn try_destruct_db_col(&self, db_col: Col) -> Result<Option<Self::Output>, String> {
        match db_col {
            Col::Text(value_as_string) => Ok(Some(value_as_string)),
            Col::Null(_) => Ok(None),
            _ => Err("illegal".into()),
        }
    }
}

impl DestructDbReturnCol for SchemaColumn<Vec<u8>> {
    type Output = Vec<u8>;

    fn destruct_db_col(&self, db_col: Col) -> Option<Self::Output> {
        self.try_destruct_db_col(db_col).expect(PANIC_EXPLANATION)
    }

    fn try_destruct_db_col(&self, db_col: Col) -> Result<Option<Self::Output>, String> {
        match db_col {
            Col::Blob(value_as_bytes) => Ok(Some(value_as_bytes)),
            Col::Null(_) => Ok(None),
            _ => Err("illegal".into()),
        }
    }
}

//db doesn't support null
// think i need to make all my normal impl stuff return potentially 2 variants. because null for a number is not the same as null for a string, but also
// null is not the same as an empty string.
