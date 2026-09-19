use crate::migration::helpers::{create_table_from_scratch, new_db_wrapper};
use crate::migration::schemas::version0;
use crate::migration::schema_versions::CURRENT_VERSION;

pub fn init_db(path: &str) {
    assert_eq!(
        CURRENT_VERSION,
        crate::migration::schema_versions::SchemaVersion::Version0,
        "init_db only handles version0. Update it after writing a migration."
    );
    let db = new_db_wrapper(path).unwrap();
    create_table_from_scratch(version0::entire_table(), &db).unwrap();
}
