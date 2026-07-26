use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::vm::{chunk::Chunk, value::Value};

#[derive(Debug, Clone)]
pub struct BoundMethod {
    pub receiver: Value,
    pub method: Rc<Closure>,
}

#[derive(Debug)]
pub enum UpvalueState {
    Open(usize),  // Points to the stack
    Close(Value), // Captured, transferred to the heap
}

pub type Upvalue = Rc<RefCell<UpvalueState>>;

#[derive(Debug)]
pub struct Closure {
    pub func: Rc<Function>,
    pub upvalues: Vec<Upvalue>,
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
