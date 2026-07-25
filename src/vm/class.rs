use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::vm::value::Value;

#[derive(Debug)]
pub struct ClassDef {
    pub name: String,
    pub methods: HashMap<String, Value>,
}

impl ClassDef {
    pub fn new(name: String, methods: HashMap<String, Value>) -> Self {
        Self { name, methods }
    }
}

#[derive(Debug)]
pub struct ClassInstance {
    pub def: Rc<ClassDef>,
    pub fields: HashMap<String, Value>,
}
