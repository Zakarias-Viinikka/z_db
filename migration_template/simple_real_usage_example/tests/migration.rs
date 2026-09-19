mod common;
use common::*;

use zutil_db::migration::helpers::{
    assert_migrated_db_matches_fresh_db, create_table_from_scratch,
};
use zutil_db::migration::schema_versions::{SchemaVersion, CURRENT_VERSION};
use zutil_db::migration::schemas::{current, version0};
use zutil_db::migration::update_from_old_schema::update_until_newest_version;

#[test]
fn migration_chain_reaches_current_version() {
    let db = setup_fresh_db("test_migration.sqlite");
    create_table_from_scratch(version0::entire_table(), &db).unwrap();

    let result = update_until_newest_version(SchemaVersion::Version0, &db);
    let expected_result = CURRENT_VERSION;
    assert_eq!(result, expected_result);

    assert_migrated_db_matches_fresh_db(current::entire_table(), &db);
}
