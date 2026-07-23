use std::{cell::RefCell, collections::HashMap, iter::Filter, rc::Rc};

use crate::{
    global_ctx::{GlobalContext, StringInterner},
    module::{Module, ModuleId},
    tol::diagnostic::{Label, miette_diagnostic::MietteDiagnostic, runtime::RuntimeError},
    vm::{
        chunk::Chunk,
        class::ClassInstance,
        function::Closure,
        native_functions::NativeFunction,
        opcode::OpCode,
        value::{Value, ValueError},
    },
};

pub mod chunk;
pub mod class;
pub mod function;
pub mod native_functions;
pub mod opcode;
pub mod value;

struct Frame {
    chunk: Rc<Chunk>,
    ip: usize,
    locals_base: usize,
    module_id: ModuleId,
}

pub struct VM<'gctx> {
    stack: Vec<Value>,
    frames: Vec<Frame>,
    globals: Vec<Value>,
    ctx: &'gctx mut GlobalContext,
}

impl<'gctx> VM<'gctx> {
    pub fn new(chunk: Chunk, ctx: &'gctx mut GlobalContext, module_id: ModuleId) -> Self {
        Self {
            stack: Vec::new(),
            globals: Vec::new(),
            ctx,
            frames: vec![Frame {
                chunk: Rc::new(chunk),
                ip: 0,
                locals_base: 1,
                module_id,
            }],
        }
    }

    pub fn run(&mut self) {
        self.assign_native("input".to_string(), 1, native_functions::native_input);
        self.assign_native("alis".to_string(), 1, native_functions::native_alis);

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
                    let index = self.current_frame().locals_base + index;
                    let value = self.stack[index].clone();
                    self.push(value);
                }
                op if op == OpCode::Print as u8 => {
                    let value = self.pop();
                    self.print_value(&value);
                    println!();
                }
                op if op == OpCode::Halt as u8 => {
                    break;
                }
                op if op == OpCode::Null as u8 => {
                    self.push(Value::Null);
                }
                op if op == OpCode::Call as u8 => {
                    let arity = self.read_byte();
                    self.call_function(arity, self.current_frame().module_id);
                }
                op if op == OpCode::Return as u8 => {
                    let value = self.pop();
                    self.return_from_frame(value);
                }
                op if op == OpCode::JumpIfFalse as u8 => {
                    let offset = self.read_u16() as usize;

                    match self.peek(0) {
                        Value::Bool(false) => {
                            self.current_frame_mut().ip += offset;
                        }
                        Value::Bool(true) => {}
                        _ => {
                            let current_module = self.current_module();
                            self.runtime_error("ang kondisyon dito ay tumatanggap lamang ng expresyong nagreresulta sa tipong `bool`", self.current_ip());
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
                op if op == OpCode::NewClassInst as u8 => {
                    let def = self.pop();
                    let field_count = self.read_byte() as usize;

                    let Value::ClassDef(class_def) = def else {
                        self.runtime_error(
                            "hindi isang klase ang nasa kaliwa ng `.`",
                            self.current_ip(),
                        );
                        return;
                    };

                    let mut fields = HashMap::new();
                    for _ in 0..field_count {
                        let Value::UninternedStr(field_name) = self.pop() else {
                            unreachable!()
                        };
                        let field_name = field_name.to_string();
                        let field_val = self.pop();

                        fields.insert(field_name, field_val);
                    }

                    let instance = ClassInstance {
                        def: class_def,
                        fields,
                    };
                    self.push(Value::ClassInstance(Rc::new(RefCell::new(instance))));
                }
                op if op == OpCode::GetField as u8 => {
                    let Value::UninternedStr(field_name) = self.pop() else {
                        panic!("Should be struct")
                    };

                    let Value::ClassInstance(instance) = self.pop() else {
                        self.runtime_error("hindi ito klase", self.current_ip());
                        return;
                    };

                    let value = instance
                        .borrow()
                        .fields
                        .get(field_name.as_ref())
                        .unwrap()
                        .clone();
                    self.push(value);
                }
                op if op == OpCode::SetField as u8 => {
                    let Value::UninternedStr(field_name) = self.pop() else {
                        panic!("str")
                    };

                    let Value::ClassInstance(instance) = self.pop() else {
                        unreachable!()
                    };

                    instance
                        .borrow_mut()
                        .fields
                        .insert(field_name.to_string(), self.pop());
                }
                op if op == OpCode::Closure as u8 => {
                    let index = self.read_byte() as usize;
                    let constant = self.current_chunk().get_constant(index);
                    let Value::Function(func) = constant else {
                        unreachable!()
                    };
                    let closure = Value::Closure(Rc::new(Closure { func }));
                    self.push(closure);
                }
                _ => println!("bug: unknown opcode {:#X}", opcode),
            }
        }
    }

    fn assign_native(
        &mut self,
        name: String,
        arity: usize,
        func: fn(&mut VM, &[Value]) -> Result<Value, Box<RuntimeError>>,
    ) {
        let id = self.ctx.get_native(&name);
        let native = Value::NativeFunction(Rc::new(NativeFunction { name, arity, func }));

        self.store_global(id, native);
    }

    fn concat(&mut self) {
        let rhs = self.pop();
        let lhs = self.pop();

        match (lhs, rhs) {
            (Value::Str(id1), Value::Str(id2)) => {
                let interner = self.ctx.string_interner();
                let str1 = interner.get(id1);
                let str2 = interner.get(id2);
                let format = format!("{}{}", str1, str2);

                let id = self.intern_string(&format);
                self.push(Value::Str(id));
            }
            _ => {
                let current_module = self.current_module();
                self.runtime_error(
                    "mga strings lamang ang pwede i-\"concatenate\"",
                    self.current_ip(),
                );
            }
        }
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - distance]
    }

    fn return_from_frame(&mut self, return_val: Value) {
        let frame = self.frames.pop().unwrap();
        self.stack.truncate(frame.locals_base);
        self.push(return_val);
    }

    fn call_function(&mut self, arity: u8, module_id: ModuleId) {
        let callee_index = self.stack.len() - 1 - arity as usize;

        let is_function = matches!(
            self.stack[callee_index],
            Value::NativeFunction(_) | Value::Closure(_)
        );
        if !is_function {
            let current_module = self.current_module();
            self.runtime_error("hindi paraan ang tinawag dito", self.current_ip());
            return;
        }
        let func_arity = match &self.stack[callee_index] {
            Value::Function(f) => f.arity,
            Value::NativeFunction(f) => f.arity as u8,
            _ => unreachable!(),
        };
        if func_arity != arity {
            let current_module = self.current_module();
            self.runtime_error("hindi tugmang bilang ng parametro at argumento: `{}` na bilang ng parametro at `{}` na bilang ng argumento", self.current_ip());
            return;
        }

        match self.peek(arity as usize) {
            Value::Closure(cl) => {
                let func = &cl.func;
                self.frames.push(Frame {
                    chunk: Rc::clone(&func.chunk),
                    ip: 0,
                    locals_base: self.stack.len() - arity as usize - 1,
                    module_id,
                });
            }
            Value::NativeFunction(func) => {
                let base = self.current_frame().locals_base + 1;
                let args = self.stack[base..arity as usize + base].to_vec();

                match ((func.func)(self, &args)) {
                    Ok(v) => self.push(v),
                    Err(r) => {
                        self.runtime_error(&r.message, self.current_ip());
                    }
                };
            }
            _ => unreachable!(),
        }
    }

    fn store_global(&mut self, index: usize, value: Value) {
        if index >= self.globals.len() {
            self.globals.resize(index + 1, Value::Null);
        }
        self.globals[index] = value;
    }

    fn store_local(&mut self, index: usize, value: Value) {
        let locals_base = self.current_frame().locals_base;
        let frame = self.current_frame_mut();

        if locals_base + index >= self.stack.len() {
            eprintln!("stack increased by {}", locals_base + index + 1);
            self.stack.resize(locals_base + index + 1, Value::Null);
        }
        self.stack[locals_base + index] = value;
    }

    fn binary_op(&mut self, f: impl Fn(Value, Value) -> Result<Value, ValueError>) {
        let right = self.pop();
        let left = self.pop();

        match f(left, right) {
            Ok(res) => {
                self.push(res);
            }
            Err(err) => {
                let current_module = self.current_module();
                self.runtime_error(&err.message, self.current_ip());
            }
        }
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
                print!("{}", self.ctx.string_interner().get(*id));
            }

            val => print!("{val}"),
        }
    }

    fn current_module(&self) -> &Module {
        self.ctx.module_by_id(self.current_frame().module_id)
    }

    fn current_module_mut(&mut self) -> &mut Module {
        self.ctx.module_by_id_mut(self.current_frame().module_id)
    }

    fn current_ip(&self) -> usize {
        self.current_frame().ip - 1
    }

    fn runtime_error(&mut self, message: &str, instruction: usize) {
        let span = self.current_chunk().span_of(instruction);
        let current_module = self.current_module();
        let runtime_err = RuntimeError::new(
            current_module.source_arc(),
            current_module.filename(),
            message,
            Label::new(span),
        );
        eprintln!(
            "{:?}",
            miette::Report::new(MietteDiagnostic::from(runtime_err))
        );
        self.force_stop_vm();
    }

    fn force_stop_vm(&mut self) {
        // naively
        self.frames.clear();
    }

    pub fn intern_string(&mut self, s: &str) -> usize {
        self.ctx.intern(s)
    }
}
