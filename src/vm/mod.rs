use std::rc::Rc;

use crate::{
    global_ctx::StringInterner,
    vm::{chunk::Chunk, opcode::OpCode, value::Value},
};

pub mod chunk;
pub mod function;
pub mod opcode;
pub mod value;

struct Frame {
    chunk: Rc<Chunk>,
    ip: usize,
    locals: Vec<Value>,
}

pub struct VM {
    stack: Vec<Value>,
    frames: Vec<Frame>,
    globals: Vec<Value>,
    string_interner: StringInterner,
}

impl VM {
    pub fn new(chunk: Chunk, string_interner: StringInterner) -> Self {
        Self {
            stack: Vec::new(),
            globals: Vec::new(),
            string_interner,
            frames: vec![Frame {
                chunk: Rc::new(chunk),
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
                op if op == OpCode::Concat as u8 => self.concat(),
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
                    self.print_value(&value);
                }
                op if op == OpCode::Halt as u8 => {
                    break;
                }
                op if op == OpCode::Null as u8 => {
                    self.push(Value::Null);
                }
                op if op == OpCode::Call as u8 => {
                    let arity = self.read_byte();
                    self.call_function(arity);
                }
                op if op == OpCode::Return as u8 => {
                    let value = self.pop();
                    self.return_from_frame(value);
                }
                x if x == OpCode::JumpIfFalse as u8 => {
                    let offset = self.read_u16() as usize;

                    match self.peek(0) {
                        Value::Bool(false) => {
                            self.current_frame_mut().ip += offset;
                        }
                        Value::Bool(true) => {}
                        _ => {
                            panic!("ang condition sa `kung` at `kundi` ay hindi tipong `bool`")
                        }
                    }
                }
                x if x == OpCode::Jump as u8 => {
                    let offset = self.read_u16() as usize;
                    self.current_frame_mut().ip += offset;
                }
                x if x == OpCode::Loop as u8 => {
                    let offset = self.read_u16() as usize;
                    self.current_frame_mut().ip -= offset;
                }
                _ => println!("bug: unknown opcode {:#X}", opcode),
            }
        }
    }

    fn concat(&mut self) {
        let rhs = self.pop();
        let lhs = self.pop();

        match (lhs, rhs) {
            (Value::Str(id1), Value::Str(id2)) => {
                let str1 = self.string_interner.get(id1);
                let str2 = self.string_interner.get(id2);

                let id = self.string_interner.intern(&format!("{}{}", str1, str2));
                self.push(Value::Str(id));
            }
            _ => panic!("Mga strings lamang ang pwede i concatinate"),
        }
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - distance]
    }

    fn return_from_frame(&mut self, return_val: Value) {
        self.frames.pop();
        self.push(return_val);
    }

    fn call_function(&mut self, arity: u8) {
        let callee_index = self.stack.len() - 1 - arity as usize;

        let is_function = matches!(self.stack[callee_index], Value::Function(_));
        if !is_function {
            panic!("popped value isnt a function")
        }
        let func_arity = match &self.stack[callee_index] {
            Value::Function(f) => f.arity,
            _ => unreachable!(),
        };
        if func_arity != arity {
            panic!(
                "hindi tugmang bilang ng parametro at argumento: `{}` na bilang ng parametro at `{}` na bilang ng argumento",
                func_arity, arity
            )
        }

        let mut locals = vec![Value::Null; arity as usize];
        for slot in (0..arity).rev() {
            locals[slot as usize] = self.pop();
        }

        let Value::Function(func) = self.pop() else {
            unreachable!("already verified above")
        };

        self.frames.push(Frame {
            chunk: Rc::clone(&func.chunk),
            ip: 0,
            locals,
        })
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

    fn read_u16(&mut self) -> u16 {
        let frame = self.current_frame_mut();
        let bytes = &frame.chunk.code()[frame.ip..frame.ip + 2];
        frame.ip += 2;

        u16::from_be_bytes([bytes[0], bytes[1]])
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

    // These are what gets shown when the value is to be printed.
    // Unimplemented variants are handled in `Value::fmt` function in the value module
    // as they do not need some values provided by the vm
    fn print_value(&self, value: &Value) {
        match value {
            Value::Str(id) => {
                println!("{}", self.string_interner.get(*id));
            }

            val => println!("{val}"),
        }
    }
}
