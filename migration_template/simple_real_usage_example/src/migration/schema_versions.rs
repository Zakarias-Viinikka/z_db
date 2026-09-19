#[derive(PartialEq, Clone, Copy, Debug)]
pub enum SchemaVersion {
    Version0,
}

pub const CURRENT_VERSION: SchemaVersion = SchemaVersion::Version0;
