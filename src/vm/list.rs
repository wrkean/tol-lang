use crate::vm::value::Value;

#[derive(Debug, Clone)]
pub struct List {
    pub elements: Vec<Value>,
}
