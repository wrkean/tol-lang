use std::rc::Rc;

use crate::vm::chunk::Chunk;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub chunk: Rc<Chunk>,
    pub arity: u8,
}

impl Function {
    pub fn new(name: String, chunk: Chunk, arity: u8) -> Self {
        Self {
            name,
            chunk: Rc::new(chunk),
            arity,
        }
    }

    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }

    pub fn arity(&self) -> u8 {
        self.arity
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
