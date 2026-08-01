use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    iter::Filter,
    rc::Rc,
};

use crate::{
    builtins,
    global_ctx::{GlobalContext, StringInterner},
    module::{Module, ModuleId},
    tol::diagnostic::{Label, miette_diagnostic::MietteDiagnostic, runtime::RuntimeError},
    vm::{
        chunk::Chunk,
        class::{ClassDef, ClassInstance},
        function::{BoundMethod, Closure, Function, Upvalue, UpvalueState},
        list::List,
        module_obj::ModuleObj,
        opcode::OpCode,
        range::Range,
        value::{Value, ValueError},
    },
};

pub mod chunk;
pub mod class;
pub mod function;
pub mod iterators;
pub mod list;
pub mod module_obj;
pub mod native_functions;
pub mod opcode;
pub mod range;
pub mod value;

struct Frame {
    closure: Rc<Closure>,
    ip: usize,
    locals_base: usize,
    module_id: ModuleId,
}

pub struct VM {
    stack: Vec<Value>,
    frames: Vec<Frame>,
    string_interner: StringInterner,
    open_upvalues: Vec<Upvalue>,
    modules: Vec<Module>,
    loaded_modules: HashMap<ModuleId, Rc<RefCell<ModuleObj>>>,
    halt: bool,
    primitive_types: HashSet<&'static str>,
}

impl VM {
    pub fn new(
        chunk: Chunk,
        string_interner: StringInterner,
        entry_module_id: ModuleId,
        modules: Vec<Module>,
    ) -> Self {
        let closure = Rc::new(Closure {
            func: Rc::new(Function::new(
                "__paraan_una__".to_string(),
                chunk,
                0,
                0,
                entry_module_id,
            )),
            upvalues: Vec::new(),
        });

        Self {
            stack: Vec::new(),
            string_interner,
            open_upvalues: Vec::new(),
            modules,
            loaded_modules: HashMap::new(),
            halt: false,
            frames: vec![Frame {
                closure,
                ip: 0,
                locals_base: 1,
                module_id: entry_module_id,
            }],
            primitive_types: HashSet::from([
                "Lista", "Sakop", "Numero", "Lutang", "Bool", "Teksto", "Iterator",
            ]),
        }
    }

    pub fn run(&mut self) {
        while self.frames.last().is_some() {
            self.step();
            if self.halt {
                break;
            }
        }
    }

    pub(crate) fn run_until_frame_depth(&mut self, target_depth: usize) {
        while self.frames.len() > target_depth {
            self.step();
            if self.halt {
                break;
            }
            if self.frames.is_empty() {
                break;
            }
        }
    }

    fn step(&mut self) {
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
            op if op == OpCode::Dup as u8 => {
                let val = self.peek(0).clone();
                self.push(val);
            }
            op if op == OpCode::Add as u8 => self.binary_op(Value::add),
            op if op == OpCode::Concat as u8 => self.concat(),
            op if op == OpCode::Sub as u8 => self.binary_op(Value::sub),
            op if op == OpCode::Mult as u8 => self.binary_op(Value::mult),
            op if op == OpCode::Div as u8 => self.binary_op(Value::div),
            op if op == OpCode::Modulo as u8 => self.binary_op(Value::modulo),
            op if op == OpCode::EqualEq as u8 => self.binary_op(Value::eqeq),
            op if op == OpCode::NotEq as u8 => self.binary_op(Value::neq),
            op if op == OpCode::Greater as u8 => self.binary_op(Value::gt),
            op if op == OpCode::GreatEq as u8 => self.binary_op(Value::ge),
            op if op == OpCode::Lesser as u8 => self.binary_op(Value::lt),
            op if op == OpCode::LessEq as u8 => self.binary_op(Value::le),
            op if op == OpCode::LogAnd as u8 => self.binary_op(Value::and),
            op if op == OpCode::LogOr as u8 => self.binary_op(Value::or),
            op if op == OpCode::Negate as u8 => self.unary_op(Value::neg),
            op if op == OpCode::LogNot as u8 => self.unary_op(Value::not),
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
                let globals = self.current_module().globals();
                let value = globals.get(index).unwrap().clone();
                self.push(value);
            }
            op if op == OpCode::LoadLocal as u8 => {
                let index = self.read_byte() as usize;
                let index = self.current_frame().locals_base + index;
                let value = self.stack[index].clone();
                self.push(value);
            }
            op if op == OpCode::LoadUpvalue as u8 => {
                let index = self.read_byte() as usize;
                let upvalue = &self.current_frame().closure.upvalues[index];
                let val = match &*upvalue.borrow() {
                    UpvalueState::Open(slot) => self.stack[*slot].clone(),
                    UpvalueState::Close(value) => value.clone(),
                };
                self.push(val);
            }
            op if op == OpCode::StoreUpvalue as u8 => {
                let index = self.read_byte() as usize;
                let value = self.pop();
                let upvalue = &self.frames.last().unwrap().closure.upvalues[index];

                let mut state = upvalue.borrow_mut();
                match &mut *state {
                    UpvalueState::Open(slot) => {
                        self.stack[*slot] = value;
                    }
                    UpvalueState::Close(closed_val) => *closed_val = value,
                }
            }
            op if op == OpCode::Print as u8 => {
                let value = self.pop();
                self.print_value(&value);
            }
            op if op == OpCode::Halt as u8 => {
                self.halt = true;
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
            op if op == OpCode::GetField as u8 => {
                let Value::Str(field_name_id) = self.pop() else {
                    panic!("Should be string")
                };
                let base = self.pop();
                let field_name = self.get_interned_string(field_name_id);

                match &base {
                    Value::ClassInstance(instance) => {
                        if let Some(field) = instance.borrow().fields.get(field_name.as_ref()) {
                            self.push(field.clone());
                        } else if let Some(method) =
                            instance.borrow().def.methods.get(field_name.as_ref())
                        {
                            let Value::Closure(closure) = method else {
                                unreachable!()
                            };
                            let bound = BoundMethod {
                                receiver: base.clone(),
                                method: closure.clone(),
                            };
                            self.push(Value::BoundMethod(Rc::new(bound)));
                        } else {
                            self.runtime_error(
                                &format!("hindi mahanap na miyembro: `{}`", field_name),
                                self.current_ip(),
                            );
                        }
                    }
                    Value::ClassDef(def) => match def.methods.get(field_name.as_ref()) {
                        Some(method) => {
                            self.push(method.clone());
                        }
                        None => {
                            self.runtime_error(
                                &format!(
                                    "walang \"method\" na `{}` ang `{}`",
                                    field_name, def.name
                                ),
                                self.current_ip(),
                            );
                        }
                    },
                    Value::ModuleObj(module_obj) => {
                        match module_obj.borrow().exports.get(field_name.as_ref()) {
                            Some(v) => self.push(v.clone()),
                            None => {
                                self.runtime_error(
                                    &format!(
                                        "walang \"method\" na `{}` ang module na `{}`",
                                        field_name,
                                        module_obj.borrow().name
                                    ),
                                    self.current_ip(),
                                );
                            }
                        }
                    }
                    // TODO: Support all values soon, we need to support calls like
                    // `1.maging_string()`
                    val => todo!("hindi pa naka support ang method na ang base ay {}", val),
                }
            }
            op if op == OpCode::SetField as u8 => {
                let Value::Str(field_name_id) = self.pop() else {
                    panic!("str")
                };
                let field_name = self.get_interned_string(field_name_id);

                let base = self.pop();

                match &base {
                    Value::ClassInstance(instance) => {
                        instance
                            .borrow_mut()
                            .fields
                            .insert(field_name.to_string(), self.pop());
                    }
                    Value::ModuleObj(module_obj) => {
                        module_obj
                            .borrow_mut()
                            .exports
                            .insert(field_name.to_string(), self.pop());
                    }
                    _ => todo!(),
                }
            }
            op if op == OpCode::Closure as u8 => {
                let index = self.read_byte() as usize;
                let constant = self.current_chunk().get_constant(index);
                let Value::Function(func) = constant else {
                    unreachable!()
                };

                let upvalue_count = self.read_byte() as usize;
                let mut upvalues = Vec::with_capacity(upvalue_count);

                for _ in 0..upvalue_count {
                    let is_local = self.read_byte() == 1;
                    let index = self.read_byte() as usize;

                    if is_local {
                        let stack_slot = self.current_frame().locals_base + index;
                        upvalues.push(self.capture_upvalue(stack_slot));
                    } else {
                        // Inherit upvalue from the current frame's closure
                        let current_closure = &self.current_frame().closure;
                        upvalues.push(current_closure.upvalues[index].clone());
                    }
                }

                let closure = Value::Closure(Rc::new(Closure { func, upvalues }));
                self.push(closure);
            }
            op if op == OpCode::Method as u8 => {
                let Value::Closure(cl) = self.pop() else {
                    unreachable!()
                };
                let Value::ClassDef(def) = self.peek(0) else {
                    unreachable!()
                };
            }
            op if op == OpCode::DefineClass as u8 => {
                let Value::Str(class_name_id) = self.pop() else {
                    unreachable!()
                };
                let class_name = self.get_interned_string(class_name_id);
                let methods_count = self.read_byte() as usize;
                let mut methods: HashMap<String, Value> = (0..methods_count)
                    .map(|_| {
                        let method = self.pop();
                        let Value::Str(name_id) = self.pop() else {
                            unreachable!()
                        };
                        let name = self.get_interned_string(name_id);

                        (name.to_string(), method)
                    })
                    .collect();
                let def = ClassDef::new(class_name.to_string(), methods);
                self.push(Value::ClassDef(Rc::new(def)));
            }
            op if op == OpCode::Invoke as u8 => {
                let const_index = self.read_byte() as usize;
                let arg_count = self.read_byte() as usize;
                let Value::Str(method_name_id) = self.current_chunk().get_constant(const_index)
                else {
                    unreachable!()
                };
                let method_name = self.get_interned_string(method_name_id);

                let receiver = self.peek(arg_count - 1).clone();
                self.invoke_method(&receiver, method_name.as_ref(), arg_count);
            }
            op if op == OpCode::List as u8 => {
                let element_count = self.read_u16();

                let mut elements = Vec::new();
                for _ in (0..element_count) {
                    elements.push(self.pop());
                }

                let list = Value::List(Rc::new(RefCell::new(List { elements })));
                self.push(list);
            }
            op if op == OpCode::ListWithCapacity as u8 => {
                let init_value = self.pop();
                let Value::Int(capacity) = self.pop() else {
                    unreachable!()
                };

                if capacity < 1 {
                    self.runtime_error(
                        "dapat ang kapasidad ay mas mahigit pa sa 0",
                        self.current_ip(),
                    );
                    return;
                }

                let mut elements = vec![init_value; capacity as usize];
                let list = Value::List(Rc::new(RefCell::new(List { elements })));
                self.push(list);
            }
            op if op == OpCode::IndexGet as u8 => {
                let index = self.pop();
                let Value::Int(index) = index else {
                    self.runtime_error("umaasa ng numero dito", self.current_ip());
                    return;
                };

                let target = self.pop();

                match target {
                    Value::List(list) => {
                        let len = list.borrow().elements.len();
                        match self.resolve_index(len, index) {
                            Some(i) => {
                                let val = list.borrow().elements[i].clone();
                                self.push(val);
                            }
                            None => {
                                self.runtime_error(
                                        &format!("mas malaki o kaparehas ang \"index\" na naibigay ({}) kesa sa bilang ng mga elemento ({})", index, len),
                                        self.current_ip(),
                                    );
                            }
                        }
                    }
                    Value::Str(id) => {
                        let string = self.get_interned_string(id);
                        let bytes = string.as_bytes();
                        match self.resolve_index(bytes.len(), index) {
                            Some(i) => {
                                let character = bytes[i] as char;
                                let id = self.intern_string(&character.to_string());
                                self.push(Value::Str(id));
                            }
                            None => {
                                self.runtime_error(
                                        &format!("mas malaki o kaparehas ang \"index\" na naibigay ({}) kesa sa bilang ng mga elemento ({})", index, bytes.len()),
                                        self.current_ip(),
                                    );
                            }
                        }
                    }
                    _ => todo!(),
                }
            }
            op if op == OpCode::IndexSet as u8 => {
                let index = self.pop();
                let target = self.pop();
                let set_val = self.pop();
                let Value::Int(index) = index else {
                    self.runtime_error("umaasa ng numero dito", self.current_ip());
                    return;
                };

                match target {
                    Value::List(list) => {
                        let len = list.borrow().elements.len();
                        match self.resolve_index(len, index) {
                            Some(i) => {
                                list.borrow_mut().elements[i] = set_val;
                            }
                            None => {
                                self.runtime_error(
                                        &format!("mas malaki o kaparehas ang \"index\" na naibigay ({}) kesa sa bilang ng mga elemento ({})", index, len),
                                        self.current_ip(),
                                    );
                            }
                        }
                    }
                    _ => todo!(),
                }
            }
            op if op == OpCode::ImportModule as u8 => {
                let constant_index = self.read_byte() as usize;
                let Value::Int(module_id) = self.current_chunk().get_constant(constant_index)
                else {
                    unreachable!()
                };
                self.run_module(module_id as usize);
                let module_obj = self.register_module(module_id as usize);
                self.push(Value::ModuleObj(module_obj));
            }
            op if op == OpCode::GetIter as u8 => {
                let iterable = self.pop();
                self.push(Value::Null);
                self.push(iterable.clone());

                // If it is iterable, call iter. If not, it may be an iterator already. Proceed and let the next instruction check
                // if it is indeed an iterator
                if iterable.is_iterable() {
                    self.invoke_method(&iterable, "iter", 1);
                }
            }
            op if op == OpCode::Bawat as u8 => {
                let target = self.read_u16();
                let iterator = self.peek(0).clone();
                // dbg!(&self.stack);
                let result = match iterator {
                    Value::Iterator(iter) => iter.borrow_mut().next(self).unwrap_or(Value::Null),
                    val => {
                        self.runtime_error(
                            &format!("hindi iterator ang nandito kundi: {val}"),
                            self.current_ip(),
                        );
                        return;
                    }
                };
                if let Value::Null = &result {
                    self.current_frame_mut().ip += target as usize;
                } else {
                    self.push(result)
                }
            }
            op if op == OpCode::MakeRange as u8 => {
                let end = self.pop();
                let start = self.pop();
                let inclusive = self.read_byte() == 1;

                let (start, end) = match (start, end) {
                    (Value::Int(s), Value::Int(e)) => (s, e),
                    _ => {
                        self.runtime_error(
                            "maaaring mga numero lamang ang operand ng `..` at `..=`",
                            self.current_ip(),
                        );
                        return;
                    }
                };
                let range = Range {
                    start,
                    end,
                    step: 1,
                    inclusive,
                };
                self.push(Value::Range(Rc::new(range)));
            }
            _ => println!("bug: unknown opcode {:#X}", opcode),
        };
    }

    fn invoke_method(&mut self, receiver: &Value, method_name: &str, arg_count: usize) {
        match receiver {
            Value::ClassInstance(inst) => {
                let inst = inst.clone();
                match inst.borrow().def.methods.get(method_name) {
                    Some(method) => {
                        // Insert at the beginning of the locals the callee itself
                        let callee_idx = self.stack.len() - arg_count - 1;
                        self.stack[callee_idx] = method.clone();
                        self.call_value(method, arg_count as u8);
                    }
                    None => {
                        self.runtime_error(
                            &format!(
                                "hindi \"method\" ng {} ang {}",
                                inst.borrow().def.name,
                                method_name
                            ),
                            self.current_ip(),
                        );
                    }
                }
            }
            Value::ClassDef(def) => {
                let def = def.clone();
                let Some(method) = def.methods.get(method_name) else {
                    self.runtime_error(
                        &format!("hindi \"method\" ng {} ang {}", def.name, method_name),
                        self.current_ip(),
                    );
                    return;
                };

                // Replace the 'Null' slot at index (len - arg_count - 1) with the method
                let callee_idx = self.stack.len() - arg_count - 1;
                self.stack[callee_idx] = method.clone();

                // Remove the ClassDef receiver from the stack
                self.stack.remove(self.stack.len() - arg_count);

                // Call the method with only the actual arguments (arg_count - 1)
                self.call_value(method, arg_count as u8 - 1);
            }
            Value::ModuleObj(module_obj) => {
                let Some(func) = module_obj.borrow().exports.get(method_name).cloned() else {
                    self.runtime_error(
                        &format!(
                            "walang paraan na `{}` ang module na `{}`",
                            method_name,
                            module_obj.borrow().name,
                        ),
                        self.current_ip(),
                    );
                    return;
                };

                // Replace the 'Null' slot at index (len - arg_count - 1) with the method
                let callee_idx = self.stack.len() - arg_count - 1;
                self.stack[callee_idx] = func.clone();

                // Remove the ModuleObj receiver from the stack
                self.stack.remove(self.stack.len() - arg_count);

                // Call the function with only the actual arguments (arg_count - 1)
                self.call_value(&func, arg_count as u8 - 1);
            }
            // Resort to calling the builtins
            val => {
                let mut args = vec![Value::Null; arg_count];
                for i in (0..arg_count).rev() {
                    args[i] = self.pop();
                }
                self.pop(); // Pops the null value
                match self.invoke_builtin_method(val, method_name, args) {
                    Ok(v) => self.push(v),
                    Err(err) => {
                        self.runtime_error(&err.message, self.current_ip());
                    }
                }
            }
        }
    }

    fn run_module(&mut self, module_id: ModuleId) {
        // Do not rerun already loaded module
        if self.loaded_modules.contains_key(&module_id) {
            return;
        }

        let module = &self.modules[module_id];
        let module_name = module.filename();
        let module_name = module_name.strip_suffix(".tol").unwrap();
        let chunk = self.modules[module_id].take_chunk();
        let closure = Closure {
            func: Rc::new(Function {
                name: format!("__{}_paraan_una__", module_name),
                chunk: Rc::new(chunk),
                arity: 0,
                frame_size: 0,
                module_id,
            }),
            upvalues: Vec::new(),
        };

        let target_depth = self.frames.len();
        self.new_frame(Rc::new(closure), self.stack.len(), 0, module_id);

        self.run_until_frame_depth(target_depth);
    }

    fn register_module(&mut self, module_id: ModuleId) -> Rc<RefCell<ModuleObj>> {
        if self.loaded_modules.contains_key(&module_id) {
            return self.loaded_modules[&module_id].clone();
        }

        let module = &mut self.modules[module_id];
        let global_name_map = module.take_global_name_map();

        let module = &self.modules[module_id];
        let exports = global_name_map
            .into_iter()
            .map(|(name, global_id)| (name, module.globals()[global_id].clone()))
            .collect::<HashMap<_, _>>();
        let module_name = module.filename().strip_suffix(".tol").unwrap().to_string();

        let module_obj = ModuleObj {
            name: module_name,
            exports,
        };
        let module_obj = Rc::new(RefCell::new(module_obj));
        self.loaded_modules.insert(module_id, module_obj.clone());

        module_obj
    }

    // Resolves a possibly-negative index into a bounds-checked usize, or None if out of range.
    fn resolve_index(&self, len: usize, index: i64) -> Option<usize> {
        let resolved = if index < 0 { index + len as i64 } else { index };
        if resolved < 0 || resolved >= len as i64 {
            None
        } else {
            Some(resolved as usize)
        }
    }

    fn invoke_builtin_method(
        &mut self,
        value: &Value,
        method_name: &str,
        args: Vec<Value>,
    ) -> Result<Value, Box<RuntimeError>> {
        let class_name = match value {
            Value::List(_) => "Lista",
            Value::Int(_) => "Numero",
            Value::Str(_) => "Teksto",
            Value::Range(_) => "Sakop",
            Value::Iterator(_) => "Iterator",
            _ => todo!(),
        };

        // This is to prevent inconsistent naming since the names of the primitive types are not
        // centered on one source
        assert!(self.primitive_types.contains(class_name));

        let def_id = *self
            .current_module()
            .global_name_map()
            .get(class_name)
            .unwrap();
        let def = self.current_module().globals().get(def_id).unwrap();
        let Value::ClassDef(def) = def else {
            unreachable!()
        };
        let native_fn = match def.methods.get(method_name) {
            Some(func) => {
                let Value::NativeFunction(func) = func else {
                    unreachable!()
                };
                func.clone()
            }
            None => {
                return Err(Box::new(self.new_runtime_error(
                    &format!("walang \"method\" na `{}` ang {}", method_name, class_name),
                    self.current_ip(),
                )));
            }
        };

        (native_fn.func)(self, &args)
    }

    fn capture_upvalue(&mut self, stack_location: usize) -> Upvalue {
        let get_location = |uv: &Upvalue| match *uv.borrow() {
            UpvalueState::Open(loc) => loc,
            _ => unreachable!(),
        };

        // Maintain open_upvalues in descending order of stack location
        match self
            .open_upvalues
            .binary_search_by(|uv| get_location(uv).cmp(&stack_location).reverse())
        {
            Ok(idx) => self.open_upvalues[idx].clone(),
            Err(idx) => {
                let new_upvalue = Rc::new(RefCell::new(UpvalueState::Open(stack_location)));
                self.open_upvalues.insert(idx, new_upvalue.clone());
                new_upvalue
            }
        }
    }

    fn concat(&mut self) {
        let rhs = self.pop();
        let lhs = self.pop();

        match (lhs, rhs) {
            (Value::Str(id1), Value::Str(id2)) => {
                let str1 = self.get_interned_string(id1);
                let str2 = self.get_interned_string(id2);
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
        self.close_upvalues(frame.locals_base);
        self.stack.truncate(frame.locals_base);
        self.push(return_val);
    }

    fn close_upvalues(&mut self, last_slot: usize) {
        let get_location = |uv: &Upvalue| match *uv.borrow() {
            UpvalueState::Open(loc) => loc,
            _ => unreachable!(),
        };

        let mut close_count = 0;
        for uv in &self.open_upvalues {
            if get_location(uv) >= last_slot {
                close_count += 1;
            } else {
                break;
            }
        }

        for uv in self.open_upvalues.drain(0..close_count) {
            let mut state = uv.borrow_mut();
            if let UpvalueState::Open(location) = *state {
                *state = UpvalueState::Close(self.stack[location].clone());
            }
        }
    }

    fn call_function(&mut self, arity: u8) {
        let callee_index = self.stack.len() - 1 - arity as usize;

        let is_function = matches!(
            self.stack[callee_index],
            Value::NativeFunction(_)
                | Value::Closure(_)
                | Value::ClassDef(_)
                | Value::BoundMethod(_)
        );
        if !is_function {
            let current_module = self.current_module();
            self.runtime_error("hindi paraan ang tinawag dito", self.current_ip());
            return;
        }
        let func_arity = match &self.stack[callee_index] {
            Value::Closure(f) => Some(f.func.arity),
            Value::NativeFunction(f) => f.arity.map(|arity| arity as u8),
            Value::ClassDef(c) => Some(0),
            Value::BoundMethod(method) => Some(method.method.func.arity - 1),
            _ => unreachable!(),
        };

        if let Some(f_arity) = func_arity
            && f_arity != arity
        {
            let current_module = self.current_module();
            self.runtime_error(&format!("hindi tugmang bilang ng parametro at argumento: ({f_arity}) na bilang ng parametro at ({arity}) na bilang ng argumento"), self.current_ip());
            return;
        }

        match self.peek(arity as usize) {
            Value::Closure(cl) => {
                self.new_frame(
                    Rc::clone(cl),
                    callee_index,
                    cl.func.frame_size,
                    cl.func.module_id,
                );
            }
            Value::NativeFunction(func) => {
                let func = func.clone();
                let mut args = vec![Value::Null; arity as usize];
                for i in (0..arity).rev() {
                    args[i as usize] = self.pop();
                }
                self.pop(); // Pop the native function itself

                match ((func.func)(self, &args)) {
                    Ok(v) => self.push(v),
                    Err(r) => {
                        self.runtime_error(&r.message, self.current_ip());
                    }
                };
            }
            Value::ClassDef(c) => {
                // Don't allow something like `Lista()` because this produces a class definition
                // Lista instead of an actual Lista object (List)!
                // TODO: Maybe allow this later? A workaround?
                if self.primitive_types.contains(c.name.as_str()) {
                    self.runtime_error(
                        "hindi pwede tawagin ang isang primitive type",
                        self.current_ip(),
                    );
                    return;
                }

                let new_instance = ClassInstance {
                    def: Rc::clone(c),
                    fields: HashMap::new(),
                };
                self.push(Value::ClassInstance(Rc::new(RefCell::new(new_instance))));
            }
            Value::BoundMethod(bound) => {
                let bound = Rc::clone(bound);
                // Put receiver at Slot 1 (1st argument position for `ako`)
                self.stack.insert(callee_index + 1, bound.receiver.clone());
                // Put method closure at Slot 0 (the callee position)
                self.stack[callee_index] = Value::Closure(Rc::clone(&bound.method));

                self.new_frame(
                    Rc::clone(&bound.method),
                    callee_index,
                    bound.method.func.frame_size,
                    bound.method.func.module_id,
                );
            }
            _ => unreachable!(),
        }
    }

    fn call_value(&mut self, value: &Value, arity: u8) {
        let is_function = matches!(
            &value,
            Value::NativeFunction(_)
                | Value::Closure(_)
                | Value::ClassDef(_)
                | Value::BoundMethod(_)
        );
        if !is_function {
            let current_module = self.current_module();
            self.runtime_error(
                &format!("hindi paraan ang tinawag dito kundi {}", value),
                self.current_ip(),
            );
            return;
        }
        let func_arity = match value {
            Value::Closure(f) => Some(f.func.arity),
            Value::NativeFunction(f) => f.arity.map(|arity| arity as u8),
            Value::ClassDef(c) => Some(0),
            Value::BoundMethod(method) => Some(method.method.func.arity - 1),
            _ => unreachable!(),
        };

        if let Some(f_arity) = func_arity
            && f_arity != arity
        {
            let current_module = self.current_module();
            self.runtime_error(&format!("hindi tugmang bilang ng parametro at argumento: ({f_arity}) na bilang ng parametro at ({arity}) na bilang ng argumento"), self.current_ip());
            return;
        }

        match value {
            Value::Closure(cl) => {
                let locals_base = self.stack.len() - arity as usize - 1;
                self.new_frame(
                    Rc::clone(cl),
                    locals_base,
                    cl.func.frame_size,
                    cl.func.module_id,
                );
            }
            Value::NativeFunction(func) => {
                let base = self.stack.len() - arity as usize;
                let mut args = vec![Value::Null; arity as usize];
                for i in (0..arity).rev() {
                    args[i as usize] = self.pop();
                }
                self.pop(); // Pop the native function itself

                match ((func.func)(self, &args)) {
                    Ok(v) => self.push(v),
                    Err(r) => {
                        self.runtime_error(&r.message, self.current_ip());
                    }
                };
            }
            Value::ClassDef(c) => {
                let new_instance = ClassInstance {
                    def: Rc::clone(c),
                    fields: HashMap::new(),
                };
                self.push(Value::ClassInstance(Rc::new(RefCell::new(new_instance))));
            }
            Value::BoundMethod(bound) => {
                let insert_at = self.stack.len() - arity as usize;
                self.stack.insert(insert_at, bound.receiver.clone());
                let closure_val = Value::Closure(Rc::clone(&bound.method));
                self.call_value(&closure_val, arity + 1);
            }
            _ => unreachable!(),
        }
    }

    fn store_global(&mut self, index: usize, value: Value) {
        let globals = self.current_module_mut().globals_mut();
        if index >= globals.len() {
            globals.resize(index + 1, Value::Null);
        }

        globals[index] = value;
    }

    fn store_local(&mut self, index: usize, value: Value) {
        let locals_base = self.current_frame().locals_base;
        let frame = self.current_frame_mut();

        if locals_base + index >= self.stack.len() {
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

    fn unary_op(&mut self, f: impl Fn(Value) -> Result<Value, ValueError>) {
        let operand = self.pop();

        match f(operand) {
            Ok(res) => {
                self.push(res);
            }
            Err(err) => {
                let current_module = self.current_module();
                self.runtime_error(&err.message, self.current_ip());
            }
        }
    }

    fn new_frame(
        &mut self,
        closure: Rc<Closure>,
        locals_base: usize,
        frame_size: usize,
        module_id: ModuleId,
    ) {
        self.stack.resize(locals_base + frame_size, Value::Null);
        self.frames.push(Frame {
            closure,
            ip: 0,
            locals_base,
            module_id,
        })
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("stack underflow")
    }

    fn read_byte(&mut self) -> u8 {
        let frame = self.current_frame_mut();
        let byte = frame.closure.func.chunk.get_byte(frame.ip);
        frame.ip += 1;

        byte
    }

    fn read_u16(&mut self) -> u16 {
        let frame = self.current_frame_mut();
        let bytes = &frame.closure.func.chunk.code()[frame.ip..frame.ip + 2];
        frame.ip += 2;

        u16::from_be_bytes([bytes[0], bytes[1]])
    }

    fn current_frame_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }

    fn current_frame(&self) -> &Frame {
        self.frames.last().unwrap()
    }

    pub fn current_chunk(&self) -> &Chunk {
        &self.current_frame().closure.func.chunk
    }

    // These are what gets shown when the value is to be printed.
    // Unimplemented variants are handled in `Value::fmt` function in the value module
    // as they do not need some values provided by the vm
    pub fn print_value(&self, value: &Value) {
        match value {
            Value::Str(id) => {
                print!("{}", self.get_interned_string(*id));
            }

            val => print!("{val}"),
        }
    }

    pub fn current_module(&self) -> &Module {
        &self.modules[self.current_frame().module_id]
    }

    pub fn current_module_mut(&mut self) -> &mut Module {
        let module_id = self.current_frame().module_id;
        &mut self.modules[module_id]
    }

    pub fn current_ip(&self) -> usize {
        self.current_frame().ip - 1
    }

    pub fn runtime_error(&mut self, message: &str, instruction: usize) {
        self.halt = true;
        let runtime_err = self.new_runtime_error(message, instruction);
        eprintln!(
            "{:?}",
            miette::Report::new(MietteDiagnostic::from(runtime_err))
        );
    }

    pub fn new_runtime_error(&self, message: &str, instruction: usize) -> RuntimeError {
        let span = self.current_chunk().span_of(instruction);
        let current_module = self.current_module();

        RuntimeError::new(
            current_module.source_arc(),
            current_module.filename(),
            message,
            Label::new(span),
        )
    }

    pub fn intern_string(&mut self, s: &str) -> usize {
        self.string_interner.intern(s)
    }

    pub fn get_interned_string(&self, id: usize) -> Rc<str> {
        self.string_interner.get(id).clone()
    }
}
