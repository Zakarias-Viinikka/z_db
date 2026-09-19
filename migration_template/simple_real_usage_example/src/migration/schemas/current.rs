use crate::migration::helpers::ADbTable;
use crate::migration::schema_versions::{SchemaVersion, CURRENT_VERSION};
use crate::migration::schemas::version0;

pub fn entire_table() -> Vec<ADbTable> {
    match CURRENT_VERSION {
        SchemaVersion::Version0 => version0::entire_table(),
    }
}
