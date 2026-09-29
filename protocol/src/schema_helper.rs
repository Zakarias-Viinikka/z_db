use std::marker::PhantomData;

use crate::row_col::{self, Col};

//enum for columns
// method for destructing that takes the enum + col to destruct
// get_colum_name that the enum "points" to
// get_type cuz why not? might be useful

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
    pub _marker: PhantomData<T>,
}

pub trait DestructDbReturnCol {
    type Output;
    fn destruct_db_col(&self, db_col: Col) -> Result<Self::Output, String>;
}

impl DestructDbReturnCol for SchemaColumn<i64> {
    type Output = i64;
    fn destruct_db_col(&self, db_col: Col) -> Result<Self::Output, String> {
        match db_col {
            Col::Integer(value_as_i64) => Ok(value_as_i64),
            _ => Err("illegal".into()),
        }
    }
}

impl DestructDbReturnCol for SchemaColumn<f64> {
    type Output = f64;
    fn destruct_db_col(&self, db_col: Col) -> Result<Self::Output, String> {
        match db_col {
            Col::Real(value_as_f64) => Ok(value_as_f64),
            _ => Err("illegal".into()),
        }
    }
}

impl DestructDbReturnCol for SchemaColumn<String> {
    type Output = String;
    fn destruct_db_col(&self, db_col: Col) -> Result<Self::Output, String> {
        match db_col {
            Col::Text(value_as_string) => Ok(value_as_string),
            _ => Err("illegal".into()),
        }
    }
}

impl DestructDbReturnCol for SchemaColumn<Vec<u8>> {
    type Output = Vec<u8>;
    fn destruct_db_col(&self, db_col: Col) -> Result<Self::Output, String> {
        match db_col {
            Col::Blob(value_as_bytes) => Ok(value_as_bytes),
            _ => Err("illegal".into()),
        }
    }
}
