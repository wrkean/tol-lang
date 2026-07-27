#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(usize),
    Function(Rc<Function>),
    Closure(Rc<Closure>),
    NativeFunction(Rc<NativeFunction>),
    ClassDef(Rc<ClassDef>),
    ClassInstance(Rc<RefCell<ClassInstance>>),
    BoundMethod(Rc<BoundMethod>),
    List(Rc<RefCell<List>>),
    Null,
}

use std::{cell::RefCell, fmt, rc::Rc, sync::Arc};

use Value::*;

use crate::{
    tol::diagnostic::runtime::RuntimeError,
    vm::{
        class::{ClassDef, ClassInstance},
        function::{BoundMethod, Closure, Function},
        list::List,
        native_functions::NativeFunction,
    },
};
impl Value {
    pub fn add(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (Int(a), Int(b)) => Ok(Int(a + b)),
            (Float(a), Float(b)) => Ok(Float(a + b)),
            (l, r) => Err(ValueError::new(format!(
                "hindi pwede ang `+` sa {l} at {r}"
            ))),
        }
    }

    pub fn sub(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (Int(a), Int(b)) => Ok(Int(a - b)),
            (Float(a), Float(b)) => Ok(Float(a - b)),
            (l, r) => Err(ValueError::new(format!(
                "hindi pwede ang `-` sa {l} at {r}"
            ))),
        }
    }

    pub fn mult(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (Int(a), Int(b)) => Ok(Int(a * b)),
            (Float(a), Float(b)) => Ok(Float(a * b)),
            (l, r) => Err(ValueError::new(format!(
                "hindi pwede ang `*` sa {l} at {r}"
            ))),
        }
    }

    pub fn div(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (_, Float(0.0)) | (_, Int(0)) => {
                Err(ValueError::new("bawal mag-\"divide\" gamit ang zero (0)"))
            }
            (Int(a), Int(b)) => Ok(Int(a / b)),
            (Float(a), Float(b)) => Ok(Float(a / b)),
            (l, r) => Err(ValueError::new(format!(
                "hindi pwede ang `/` sa {l} at {r}"
            ))),
        }
    }

    pub fn eqeq(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (Int(a), Int(b)) => Ok(Bool(a == b)),
            (Float(a), Float(b)) => Ok(Bool(a == b)),
            (Bool(a), Bool(b)) => Ok(Bool(a == b)),
            (Str(a), Str(b)) => Ok(Bool(a == b)),
            (l, r) => Err(ValueError::new(format!(
                "hindi pwede ang `==` sa {l} at {r}"
            ))),
        }
    }

    pub fn neq(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (Int(a), Int(b)) => Ok(Bool(a != b)),
            (Float(a), Float(b)) => Ok(Bool(a != b)),
            (Bool(a), Bool(b)) => Ok(Bool(a != b)),
            (Str(a), Str(b)) => Ok(Bool(a != b)),
            (l, r) => Err(ValueError::new(format!(
                "hindi pwede ang `!=` sa {l} at {r}"
            ))),
        }
    }

    pub fn gt(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (Int(a), Int(b)) => Ok(Bool(a > b)),
            (Float(a), Float(b)) => Ok(Bool(a > b)),
            (l, r) => Err(ValueError::new(format!(
                "hindi pwede ang `>` sa {l} at {r}"
            ))),
        }
    }

    pub fn ge(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (Int(a), Int(b)) => Ok(Bool(a >= b)),
            (Float(a), Float(b)) => Ok(Bool(a >= b)),
            (l, r) => Err(ValueError::new(format!(
                "hindi pwede ang `>=` sa {l} at {r}"
            ))),
        }
    }

    pub fn lt(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (Int(a), Int(b)) => Ok(Bool(a < b)),
            (Float(a), Float(b)) => Ok(Bool(a < b)),
            (l, r) => Err(ValueError::new(format!(
                "hindi pwede ang `<` sa {l} at {r}"
            ))),
        }
    }

    pub fn le(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (Int(a), Int(b)) => Ok(Bool(a <= b)),
            (Float(a), Float(b)) => Ok(Bool(a <= b)),
            (l, r) => Err(ValueError::new(format!(
                "hindi pwede ang `<=` sa {l} at {r}"
            ))),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // These are what gets shown when the value is to be printed.
        // Unimplemented variants are handled in `VM::print_value` function in the VM
        // as they need some values provided only by the vm
        match self {
            Int(x) => write!(f, "{x}"),
            Float(x) => write!(f, "{x}"),
            Bool(true) => write!(f, "totoo"),
            Bool(false) => write!(f, "mali"),
            Null => write!(f, "<WALA>"),
            Function(func) => write!(f, "<paraan '{}'>", func.name),
            Closure(cl) => write!(f, "<paraan '{}'>", cl.func.name),
            NativeFunction(func) => write!(f, "<native_paraan '{}'>", func.name),
            Str(s) => write!(f, "<string id={s}>"),
            ClassDef(def) => write!(f, "<klase_def '{}'>", def.name),
            ClassInstance(inst) => write!(f, "<klase_inst '{}'>", inst.borrow().def.name),
            BoundMethod(method) => write!(f, "<method '{}'>", method.method.func.name),
            List(list) => {
                write!(
                    f,
                    "[{}]",
                    list.borrow()
                        .elements
                        .iter()
                        .map(|val| val.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

pub struct ValueError {
    pub message: String,
    pub help: Option<String>,
}

impl ValueError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            help: None,
        }
    }

    pub fn help(mut self, message: impl Into<String>) -> Self {
        self.help = Some(message.into());

        self
    }
}
