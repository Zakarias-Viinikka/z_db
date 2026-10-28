use js_sys::Array;
use protocol::error::DbError;
use protocol::payload::*;
use protocol::serialization::Convert;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    fn javascript_im_begging_you(args: &JsValue) -> js_sys::Promise;
}

async fn tell_worker_to_do(cmd: &str, payload: Option<Vec<u8>>) -> Result<Vec<u8>, DbError> {
    let arr = Array::new();
    arr.push(&JsValue::from_str(cmd));

    if let Some(bytes) = payload {
        arr.push(&js_sys::Uint8Array::from(&bytes[..]));
    }

    let promise = javascript_im_begging_you(&arr);
    let raw = JsFuture::from(promise)
        .await
        .map_err(|e| DbError::CureFail(format!("promise rejected: {:?}", e)))?;
    let result_in_js_form: Array = raw
        .dyn_into()
        .map_err(|_| DbError::CureFail("response was not an array".into()))?;

    if response_is_error(&result_in_js_form) {
        return Err(extract_error(&result_in_js_form));
    }

    let js_form_of_rust_vecu8: js_sys::Uint8Array = result_in_js_form
        .get(1)
        .dyn_into()
        .map_err(|_| DbError::CureFail("response[1] was not a Uint8Array".into()))?;
    let mut rust_vecu8_with_same_length_as_js_version_of_vecu8_but_every_u8_is_zero =
        vec![0u8; js_form_of_rust_vecu8.length() as usize];
    js_form_of_rust_vecu8
        .copy_to(&mut rust_vecu8_with_same_length_as_js_version_of_vecu8_but_every_u8_is_zero);
    let vecu8_final = rust_vecu8_with_same_length_as_js_version_of_vecu8_but_every_u8_is_zero;
    Ok(vecu8_final)
}

fn response_is_error(result_in_js_form: &Array) -> bool {
    result_in_js_form
        .get(0)
        .as_string()
        .map(|s| s == "error")
        .unwrap_or(false)
}

fn extract_error(result_in_js_form: &Array) -> DbError {
    let msg = result_in_js_form
        .get(1)
        .as_string()
        .unwrap_or_else(|| "unknown worker error".into());
    DbError::CureFail(msg)
}

async fn no_input_no_output(cmd: &str) -> Result<(), DbError> {
    let bytes = tell_worker_to_do(cmd, None).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

async fn no_input_with_output<Out: Convert>(cmd: &str) -> Result<Out, DbError> {
    let bytes = tell_worker_to_do(cmd, None).await?;
    Out::un_payloadify(&bytes)
}

async fn input_no_output<In: Convert>(cmd: &str, input: In) -> Result<(), DbError> {
    let bytes = tell_worker_to_do(cmd, Some(input.to_payload())).await?;
    Result::<(), DbError>::un_payloadify(&bytes)?
}

async fn input_with_output<In: Convert, Out: Convert>(
    cmd: &str,
    input: In,
) -> Result<Out, DbError> {
    let bytes = tell_worker_to_do(cmd, Some(input.to_payload())).await?;
    Out::un_payloadify(&bytes)
}

// no input, output
pub async fn list_tables() -> Result<ListTablesOut, DbError> {
    no_input_with_output("list_tables").await
}

pub async fn export_database() -> Result<ExportDatabaseOut, DbError> {
    no_input_with_output("export_database").await
}

// no input, no output
pub async fn begin_all_or_nothing() -> Result<(), DbError> {
    no_input_no_output("begin_all_or_nothing").await
}

pub async fn everything_went_perfectly() -> Result<(), DbError> {
    no_input_no_output("everything_went_perfectly").await
}

pub async fn regret_everything() -> Result<(), DbError> {
    no_input_no_output("regret_everything").await
}

pub async fn are_we_in_middle_of_transaction() -> Result<AreWeInMiddleOfTransactionOut, DbError> {
    no_input_with_output("are_we_in_middle_of_transaction").await
}

// input, no output
pub async fn create_table(input: CreateTableIn) -> Result<(), DbError> {
    input_no_output("create_table", input).await
}

pub async fn drop_table(input: DropTableIn) -> Result<(), DbError> {
    input_no_output("drop_table", input).await
}

pub async fn insert_data(input: InsertDataIn) -> Result<(), DbError> {
    input_no_output("insert_data", input).await
}

pub async fn insert_data_and_get_col(
    input: InsertDataAndGetColIn,
) -> Result<InsertDataAndGetColOut, DbError> {
    input_with_output("insert_data_and_get_col", input).await
}

pub async fn delete_row(input: DeleteRowIn) -> Result<(), DbError> {
    input_no_output("delete_row", input).await
}

pub async fn delete_row_where(input: DeleteRowWhereIn) -> Result<(), DbError> {
    input_no_output("delete_row_where", input).await
}

pub async fn delete_all_rows(input: DeleteAllRowsIn) -> Result<(), DbError> {
    input_no_output("delete_all_rows", input).await
}

pub async fn edit_col_in_row(input: EditColInRowIn) -> Result<(), DbError> {
    input_no_output("edit_col_in_row", input).await
}

pub async fn edit_col_in_row_where(input: EditColInRowWhereIn) -> Result<(), DbError> {
    input_no_output("edit_col_in_row_where", input).await
}

pub async fn swap_columns(input: SwapColumnsIn) -> Result<(), DbError> {
    input_no_output("swap_columns", input).await
}

pub async fn create_index(input: CreateIndexIn) -> Result<(), DbError> {
    input_no_output("create_index", input).await
}

pub async fn add_column(input: AddColumnIn) -> Result<(), DbError> {
    input_no_output("add_column", input).await
}

pub async fn remove_column(input: RemoveColumnIn) -> Result<(), DbError> {
    input_no_output("remove_column", input).await
}

pub async fn fundamentally_edit_existing_col(
    input: FundamentallyEditExistingColIn,
) -> Result<(), DbError> {
    input_no_output("fundamentally_edit_existing_col", input).await
}

pub async fn create_table_from_export(input: CreateTableFromExportIn) -> Result<(), DbError> {
    input_no_output("create_table_from_export", input).await
}

pub async fn copy_table(input: CopyTableIn) -> Result<(), DbError> {
    input_no_output("copy_table", input).await
}

pub async fn create_foreign_table(input: CreateForeignTableIn) -> Result<(), DbError> {
    input_no_output("create_foreign_table", input).await
}

pub async fn create_fts5_table(input: CreateFts5TableIn) -> Result<(), DbError> {
    input_no_output("create_fts5_table", input).await
}

pub async fn rebuild_fts5_index(input: RebuildFts5In) -> Result<(), DbError> {
    input_no_output("rebuild_fts5_index", input).await
}

pub async fn force_drop_table(input: DropTableIn) -> Result<(), DbError> {
    input_no_output("force_drop_table", input).await
}

// input, output
pub async fn get_data(input: GetDataIn) -> Result<GetDataOut, DbError> {
    input_with_output("get_data", input).await
}

pub async fn get_data_ordered(input: GetDataOrderedIn) -> Result<GetDataOut, DbError> {
    input_with_output("get_data_ordered", input).await
}

pub async fn check_table(input: CheckTableIn) -> Result<CheckTableOut, DbError> {
    input_with_output("check_table", input).await
}

pub async fn check_index(input: CheckIndexIn) -> Result<CheckIndexOut, DbError> {
    input_with_output("check_index", input).await
}

pub async fn export_tables(input: ExportTablesIn) -> Result<ExportTablesOut, DbError> {
    input_with_output("export_tables", input).await
}

pub async fn count_all_rows(input: CountAllRowsIn) -> Result<CountAllRowsOut, DbError> {
    input_with_output("count_all_rows", input).await
}

pub async fn count_rows(input: CountRowsIn) -> Result<CountRowsOut, DbError> {
    input_with_output("count_rows", input).await
}

pub async fn search_fts5(input: SearchFts5In) -> Result<SearchFts5Out, DbError> {
    input_with_output("search_fts5", input).await
}

pub async fn get_single_col(input: GetSingleColIn) -> Result<GetSingleColOut, DbError> {
    input_with_output("get_single_col", input).await
}

pub async fn get_data_ordered_and_skip_first_x(
    input: GetDataOrderedAndSkipFirstXIn,
) -> Result<GetDataOut, DbError> {
    input_with_output("get_data_ordered_and_skip_first_x", input).await
}