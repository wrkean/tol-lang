use crate::vm::{chunk::Chunk, opcode::OpCode, value::Value};

pub mod chunk;
pub mod opcode;
pub mod value;

struct Frame {
    chunk: Chunk,
    ip: usize,
    locals: Vec<Value>,
}

pub struct VM {
    stack: Vec<Value>,
    frames: Vec<Frame>,
    globals: Vec<Value>,
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            stack: Vec::new(),
            globals: Vec::new(),
            frames: vec![Frame {
                chunk,
                ip: 0,
                locals: Vec::new(),
            }],
        }
    }

    pub fn run(&mut self) {
        while self.frames.last().is_some() {
            let opcode = self.read_byte();

            match opcode {
                op if op == OpCode::Constant as u8 => {
                    let index = self.read_byte() as usize;
                    let constant = self.current_chunk().get_constant(index);
                    self.push(constant);
                }
                op if op == OpCode::Pop as u8 => {
                    self.pop();
                }
                op if op == OpCode::Add as u8 => self.binary_op(Value::add),
                op if op == OpCode::Sub as u8 => self.binary_op(Value::sub),
                op if op == OpCode::Mult as u8 => self.binary_op(Value::mult),
                op if op == OpCode::Div as u8 => self.binary_op(Value::div),
                op if op == OpCode::EqualEq as u8 => self.binary_op(Value::eqeq),
                op if op == OpCode::NotEq as u8 => self.binary_op(Value::neq),
                op if op == OpCode::Greater as u8 => self.binary_op(Value::gt),
                op if op == OpCode::GreatEq as u8 => self.binary_op(Value::ge),
                op if op == OpCode::Lesser as u8 => self.binary_op(Value::lt),
                op if op == OpCode::LessEq as u8 => self.binary_op(Value::le),
                op if op == OpCode::StoreGlobal as u8 => {
                    let index = self.read_byte() as usize;
                    let value = self.pop();
                    self.store_global(index, value);
                }
                op if op == OpCode::StoreLocal as u8 => {
                    let index = self.read_byte() as usize;
                    let value = self.pop();
                    self.store_local(index, value);
                }
                op if op == OpCode::LoadGlobal as u8 => {
                    let index = self.read_byte() as usize;
                    let value = self.globals.get(index).unwrap().clone();
                    self.push(value);
                }
                op if op == OpCode::LoadLocal as u8 => {
                    let index = self.read_byte() as usize;
                    let value = self.current_frame().locals.get(index).unwrap().clone();
                    self.push(value);
                }
                op if op == OpCode::Print as u8 => {
                    let value = self.pop();
                    println!("{value}");
                }
                op if op == OpCode::Halt as u8 => {
                    break;
                }
                op if op == OpCode::Null as u8 => {
                    self.push(Value::Null);
                }
                _ => println!("bug: unknown opcode {:#X}", opcode),
            }
        }
    }

    fn store_global(&mut self, index: usize, value: Value) {
        if index >= self.globals.len() {
            self.globals.resize(index + 1, Value::Null);
        }
        self.globals[index] = value;
    }

    fn store_local(&mut self, index: usize, value: Value) {
        let frame = self.current_frame_mut();
        if index >= frame.locals.len() {
            frame.locals.resize(index + 1, Value::Null);
        }
        frame.locals[index] = value;
    }

    fn binary_op(&mut self, f: impl Fn(Value, Value) -> Value) {
        let right = self.pop();
        let left = self.pop();

        self.push(f(left, right));
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("stack underflow")
    }

    fn read_byte(&mut self) -> u8 {
        let frame = self.current_frame_mut();
        let byte = frame.chunk.get_byte(frame.ip);
        frame.ip += 1;

        byte
    }

    fn current_frame_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }

    fn current_frame(&self) -> &Frame {
        self.frames.last().unwrap()
    }

    fn current_chunk(&self) -> &Chunk {
        &self.current_frame().chunk
    }
}
