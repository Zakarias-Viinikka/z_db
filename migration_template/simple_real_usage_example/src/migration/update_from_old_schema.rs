use db_wrapper::mascot::LiveForever;

use crate::migration::schema_versions::SchemaVersion;

pub fn update_once(current: SchemaVersion, _db_wrapper: &LiveForever) -> SchemaVersion {
    match current {
        SchemaVersion::Version0 => SchemaVersion::Version0,
    }
}

pub fn update_until_newest_version(
    mut current: SchemaVersion,
    db_wrapper: &LiveForever,
) -> SchemaVersion {
    loop {
        let next = update_once(current, db_wrapper);
        if next == current {
            return next;
        }
        current = next;
    }
}
