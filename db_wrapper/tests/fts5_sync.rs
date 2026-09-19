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
fn sync_inserts_only_target_row() {
    let db = setup!();

    create_text_table!(&db, [id_column(), not_null_text_col("keyword"),]);

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

    create_text_fts5!(&db, ["keyword"]);

    update_single_fts5_row(UpdateSingleFts5Row {
        row_id: "2",
        old_value: None,
        new_value: Some("beta"),
        db: &db,
    });

    let result = search_row_ids(SearchRowIds {
        query: "beta",
        db: &db,
    });
    let expected_result = vec![2];
    assert_eq!(result, expected_result);

    let result = search_row_ids(SearchRowIds {
        query: "alpha",
        db: &db,
    });
    let expected_result: Vec<i64> = Vec::new();
    assert_eq!(result, expected_result);

    let result = search_row_ids(SearchRowIds {
        query: "gamma",
        db: &db,
    });
    let expected_result: Vec<i64> = Vec::new();
    assert_eq!(result, expected_result);
}

#[test]
fn sync_adds_and_removes_row_from_index() {
    let db = setup!();

    create_text_table!(&db, [id_column(), not_null_text_col("keyword"),]);

    create_text_fts5!(&db, ["keyword"]);

    // empty table, fts5 finds nothing
    let result = search_row_ids(SearchRowIds {
        query: "alpha",
        db: &db,
    });
    let expected_result: Vec<i64> = Vec::new();
    assert_eq!(result, expected_result);

    // add row, sync it, fts5 finds it
    insert_text_row(InsertTextRow {
        text: "alpha",
        db: &db,
    }); //id 1

    update_single_fts5_row(UpdateSingleFts5Row {
        row_id: "1",
        old_value: None,
        new_value: Some("alpha"),
        db: &db,
    });

    let result = search_row_ids(SearchRowIds {
        query: "alpha",
        db: &db,
    });
    let expected_result = vec![1];
    assert_eq!(result, expected_result);

    // delete row, sync it, fts5 finds nothing
    delete_text_row(DeleteTextRow {
        row_id: "1",
        db: &db,
    });

    update_single_fts5_row(UpdateSingleFts5Row {
        row_id: "1",
        old_value: Some("alpha"),
        new_value: None,
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

struct UpdateSingleFts5Row<'a> {
    row_id: &'a str,
    old_value: Option<&'a str>,
    new_value: Option<&'a str>,
    db: &'a LiveForever,
}

fn update_single_fts5_row(params: UpdateSingleFts5Row) {
    params
        .db
        .sync_fts5_row(SyncFts5RowIn {
            source_table_name: "keyword_lookup".to_string(),
            row_id: params.row_id.to_string(),
            column_name: "keyword".to_string(),
            old_value: params.old_value.map(|s| s.to_string()),
            new_value: params.new_value.map(|s| s.to_string()),
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
