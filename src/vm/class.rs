use std::collections::HashSet;

#[derive(Debug)]
pub struct ClassDef {
    pub name: String,
}

impl ClassDef {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
