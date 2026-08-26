use crate::serialization::Base64Bytes;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub message_id: usize,
    pub request: Request,
    pub payload: Base64Bytes, // bincode-serialized payload, base64-encoded in JSON
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub message_id: usize,
    pub request: Request,
    pub payload: Base64Bytes,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum Request {
    CreateTable,
    ListTables,
    GetData,
    GetDataOrdered,
    InsertData,
    DropTable,
    EditColInRow,
    CheckTable,
    DeleteRow,
    SwapColumns,
    CreateIndex,
    CheckIndex,
    AddColumn,
    RemoveColumn,
    ExportDatabase,
    ExportTables,
    CreateTableFromExport,
    CopyTable,
    UniffiDontRenameMyEnums(bool),
}
