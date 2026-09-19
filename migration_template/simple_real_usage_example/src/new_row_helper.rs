use protocol::row_col::{Col, Row};

pub fn new_row_text(title: String, body: String) -> Row {
    Row {
        cols: vec![Col::Text(title), Col::Text(body)],
    }
}

pub fn new_row_keyword_lookup(text_id: i64, keyword: String) -> Row {
    Row {
        cols: vec![Col::Integer(text_id), Col::Text(keyword)],
    }
}
