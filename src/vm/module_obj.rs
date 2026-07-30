use std::collections::HashMap;

use crate::vm::value::Value;

#[derive(Debug)]
pub struct ModuleObj {
    pub name: String,
    pub exports: HashMap<String, Value>,
}
