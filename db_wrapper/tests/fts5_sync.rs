#![cfg(feature = "testing")]

#[macro_use]
mod macros;

mod common;
use common::*;

use db_wrapper::testing_mascot::LiveForever;
use protocol::new_table::id_column;
use protocol::payload::*;
use protocol::row_col::Col;

#[test]
fn fts5_stays_in_sync_through_insert_update_delete() {
    let db = setup!();

    create_text_table!(&db, [id_column(), not_null_text_col("keyword"),]);

    create_text_fts5!(&db, ["keyword"]);

    // insert syncs
    insert_text_row(InsertTextRow {
        text: "alpha",
        db: &db,
    }); //id 1
    insert_text_row(InsertTextRow {
        text: "beta",
        db: &db,
    }); //id 2
    insert_text_row(InsertTextRow {
        text: "gamma",
        db: &db,
    }); //id 3

    let result = search_row_ids(SearchRowIds {
        query: "beta",
        db: &db,
    });
    let expected_result = vec![2];
    assert_eq!(result, expected_result);

    // update syncs
    update_text_row(UpdateTextRow {
        row_id: "2",
        new_value: "delta",
        db: &db,
    });

    let result = search_row_ids(SearchRowIds {
        query: "beta",
        db: &db,
    });
    let expected_result: Vec<i64> = Vec::new();
    assert_eq!(result, expected_result);

    let result = search_row_ids(SearchRowIds {
        query: "delta",
        db: &db,
    });
    let expected_result = vec![2];
    assert_eq!(result, expected_result);

    // delete syncs
    delete_text_row(DeleteTextRow {
        row_id: "1",
        db: &db,
    });

    let result = search_row_ids(SearchRowIds {
        query: "alpha",
        db: &db,
    });
    let expected_result: Vec<i64> = Vec::new();
    assert_eq!(result, expected_result);
}

// ============================================================
// HELPERS
// ============================================================

struct InsertTextRow<'a> {
    text: &'a str,
    db: &'a LiveForever,
}

fn insert_text_row(params: InsertTextRow) {
    params
        .db
        .insert_data(InsertDataIn {
            table_name: "keyword_lookup".to_string(),
            values: vec![ColumnValue {
                column_name: "keyword".to_string(),
                value: Col::Text(params.text.to_string()),
            }],
        })
        .unwrap();
}

struct UpdateTextRow<'a> {
    row_id: &'a str,
    new_value: &'a str,
    db: &'a LiveForever,
}

fn update_text_row(params: UpdateTextRow) {
    params
        .db
        .edit_col_in_row(EditColInRowIn {
            table_name: "keyword_lookup".to_string(),
            row_id: params.row_id.to_string(),
            column: "keyword".to_string(),
            new_value: Col::Text(params.new_value.to_string()),
        })
        .unwrap();
}

struct DeleteTextRow<'a> {
    row_id: &'a str,
    db: &'a LiveForever,
}

fn delete_text_row(params: DeleteTextRow) {
    params
        .db
        .delete_row(DeleteRowIn {
            table_name: "keyword_lookup".to_string(),
            row_id: params.row_id.to_string(),
        })
        .unwrap();
}

struct SearchRowIds<'a> {
    query: &'a str,
    db: &'a LiveForever,
}

fn search_row_ids(params: SearchRowIds) -> Vec<i64> {
    params
        .db
        .search_fts5(SearchFts5In {
            table_name: "keyword_lookup".to_string(),
            text_to_lookup: params.query.to_string(),
        })
        .unwrap()
        .rows
        .iter()
        .map(|r| match &r.cols[0] {
            Col::Integer(i) => *i,
            _ => panic!("expected integer rowid"),
        })
        .collect()
}
