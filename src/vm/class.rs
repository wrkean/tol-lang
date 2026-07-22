use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::vm::value::Value;

#[derive(Debug)]
pub struct ClassDef {
    pub name: String,
}

impl ClassDef {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Debug)]
pub struct ClassInstance {
    pub def: Rc<ClassDef>,
    pub fields: HashMap<String, Value>,
}
