#[derive(Debug, Clone)]
pub struct Snippet {
    pub file: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub snippets: Vec<Snippet>,
}
