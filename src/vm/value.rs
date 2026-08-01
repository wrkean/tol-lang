use std::{cell::RefCell, fmt, rc::Rc, sync::Arc};

use Value::*;

use crate::{
    builtins::native_functions::NativeFunction,
    tol::diagnostic::runtime::RuntimeError,
    vm::{
        VM,
        class::{ClassDef, ClassInstance},
        function::{BoundMethod, Closure, Function},
        iterators::NativeIterator,
        list::List,
        module_obj::ModuleObj,
        range::Range,
    },
};

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
    ModuleObj(Rc<RefCell<ModuleObj>>),
    Iterator(Rc<RefCell<dyn NativeIterator>>),
    Range(Rc<Range>),
    Null,
}
impl Value {
    pub fn add(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (Int(a), Int(b)) => Ok(Int(a.wrapping_add(b))),
            (Float(a), Float(b)) => Ok(Float(a + b)),
            (l, r) => Err(ValueError::new(format!(
                "hindi pwede ang `+` sa {l} at {r}"
            ))),
        }
    }

    pub fn sub(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (Int(a), Int(b)) => Ok(Int(a.wrapping_sub(b))),
            (Float(a), Float(b)) => Ok(Float(a - b)),
            (l, r) => Err(ValueError::new(format!(
                "hindi pwede ang `-` sa {l} at {r}"
            ))),
        }
    }

    pub fn mult(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (Int(a), Int(b)) => Ok(Int(a.wrapping_mul(b))),
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
            (Int(a), Int(b)) => Ok(Int(a.wrapping_div(b))),
            (Float(a), Float(b)) => Ok(Float(a / b)),
            (l, r) => Err(ValueError::new(format!(
                "hindi pwede ang `/` sa {l} at {r}"
            ))),
        }
    }

    pub fn modulo(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (_, Float(0.0)) | (_, Int(0)) => {
                Err(ValueError::new("bawal mag-\"modulo\" gamit ang zero (0)"))
            }
            (Int(a), Int(b)) => Ok(Int(a % b)),
            (Float(a), Float(b)) => Ok(Float(a % b)),
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

    pub fn and(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (Bool(b1), Bool(b2)) => Ok(Bool(b1 && b2)),
            (l, r) => Err(ValueError::new(
                "maaari lamang na mga \"boolean\" ang pwede sa `at`",
            )),
        }
    }

    pub fn or(self, right: Self) -> Result<Self, ValueError> {
        match (self, right) {
            (Bool(b1), Bool(b2)) => Ok(Bool(b1 || b2)),
            (l, r) => Err(ValueError::new(
                "maaari lamang na mga \"boolean\" ang pwede sa `o`",
            )),
        }
    }

    pub fn not(self) -> Result<Self, ValueError> {
        match (self) {
            Bool(b) => Ok(Bool(!b)),
            _ => Err(ValueError::new(
                "maaari lamang na mga \"boolean\" ang pwede sa `di`",
            )),
        }
    }

    pub fn neg(self) -> Result<Self, ValueError> {
        match self {
            Int(x) => Ok(Int(-x)),
            Float(x) => Ok(Float(-x)),
            _ => Err(ValueError::new(
                "maaari lamang na mga numero at lutang ang pwede sa unary na `-`",
            )),
        }
    }

    pub fn to_printed_string(&self, vm: &VM) -> String {
        match self {
            Int(_)
            | Float(_)
            | Bool(_)
            | Value::Function(_)
            | Value::NativeFunction(_)
            | Value::ClassDef(_)
            | Value::ClassInstance(_)
            | Value::BoundMethod(_)
            | Value::List(_)
            | Value::Closure(_)
            | Null
            | Value::ModuleObj(_)
            | Value::Range(_)
            | Value::Iterator(_) => self.to_string(),
            Value::Str(id) => vm.get_interned_string(*id).to_string(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            ModuleObj(module_obj) => write!(f, "<modyul '{}'>", module_obj.borrow().name),
            Iterator(iter) => write!(f, "<iterator>"),
            Range(r) => {
                let op_str = if r.inclusive { "..=" } else { ".." };
                write!(f, "{}{}{}..{}", r.start, op_str, r.end, r.step)
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
