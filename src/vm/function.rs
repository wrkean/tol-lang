use std::rc::Rc;

use crate::vm::chunk::Chunk;

#[derive(Debug)]
pub struct Closure {
    pub func: Rc<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub chunk: Rc<Chunk>,
    pub arity: u8,
    pub frame_size: usize,
}

impl Function {
    pub fn new(name: String, chunk: Chunk, arity: u8, frame_size: usize) -> Self {
        Self {
            name,
            chunk: Rc::new(chunk),
            arity,
            frame_size,
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
