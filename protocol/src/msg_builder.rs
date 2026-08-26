use crate::error::DbError;
use crate::messages::*;
use crate::payload::{GetDataIn, GetDataOut};
use crate::serialization::*;

#[uniffi::export]
pub fn build_get_data_in_msg(message_id: u64, pre_payload: GetDataIn) -> Result<String, DbError> {
    let bytes = pre_payload.to_payload();
    let msg = Message {
        message_id: message_id as usize,
        request: Request::GetData,
        payload: Base64Bytes(bytes),
    };
    message_to_json_str(&msg).map_err(|e| DbError::SerializeError(e.to_string()))
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GetDataOutUnbuilt {
    pub message_id: u64,
    pub request: Request,
    pub get_data_out: GetDataOut,
}

#[uniffi::export]
pub fn unbuild_get_data_out_response(response: String) -> Result<GetDataOutUnbuilt, DbError> {
    let result =
        json_str_to_response(&response).map_err(|e| DbError::SerializeError(e.to_string()))?;
    let (message_id, request, data) = (result.message_id, result.request, result.payload);

    let get_data_out = GetDataOut::un_payloadify(&data.0)?;

    Ok(GetDataOutUnbuilt {
        message_id: message_id as u64,
        request,
        get_data_out,
    })
}
