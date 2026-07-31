#[derive(Debug, Clone)]
pub struct Range {
    pub start: i64,
    pub end: i64,
    pub step: i64,
    pub inclusive: bool,
}
