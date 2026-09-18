use migration_template::helpers::{
    assert_migrated_db_matches_fresh_db, create_table_from_scratch, new_db_wrapper,
};
use migration_template::schema_versions::SchemaVersion;
use migration_template::schemas::{version0, version4};
use migration_template::update_from_old_schema::update_until_newest_version;

fn main() {
    let db_wrapper = new_db_wrapper().unwrap();

    create_table_from_scratch(version0::entire_table(), &db_wrapper).unwrap();

    update_until_newest_version(SchemaVersion::Version0, &db_wrapper);

    assert_migrated_db_matches_fresh_db(version4::entire_table(), &db_wrapper);

    println!("migration chain OK");
}
