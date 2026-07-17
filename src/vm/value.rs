#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(usize),
    Function(Rc<Function>),
    Null,
}

use std::{fmt, rc::Rc};

use Value::*;

use crate::vm::function::Function;
impl Value {
    pub fn add(self, right: Self) -> Self {
        match (self, right) {
            (Int(a), Int(b)) => Int(a + b),
            (Float(a), Float(b)) => Float(a + b),
            (l, r) => panic!("cannot add {l} and {r} together"),
        }
    }

    pub fn sub(self, right: Self) -> Self {
        match (self, right) {
            (Int(a), Int(b)) => Int(a - b),
            (Float(a), Float(b)) => Float(a - b),
            (l, r) => panic!("cannot sub {l} and {r} together"),
        }
    }

    pub fn mult(self, right: Self) -> Self {
        match (self, right) {
            (Int(a), Int(b)) => Int(a * b),
            (Float(a), Float(b)) => Float(a * b),
            (l, r) => panic!("cannot mult {l} and {r} together"),
        }
    }

    pub fn div(self, right: Self) -> Self {
        match (self, right) {
            (_, Float(0.0)) | (_, Int(0)) => panic!("cannot divide by zero"),
            (Int(a), Int(b)) => Int(a / b),
            (Float(a), Float(b)) => Float(a / b),
            (l, r) => panic!("cannot div {l} and {r} together"),
        }
    }

    pub fn eqeq(self, right: Self) -> Self {
        match (self, right) {
            (Int(a), Int(b)) => Bool(a == b),
            (Float(a), Float(b)) => Bool(a == b),
            (Bool(a), Bool(b)) => Bool(a == b),
            (Str(a), Str(b)) => Bool(a == b),
            (l, r) => panic!("cannot mult {l} and {r} together"),
        }
    }

    pub fn neq(self, right: Self) -> Self {
        match (self, right) {
            (Int(a), Int(b)) => Bool(a != b),
            (Float(a), Float(b)) => Bool(a != b),
            (Bool(a), Bool(b)) => Bool(a != b),
            (Str(a), Str(b)) => Bool(a != b),
            (l, r) => panic!("cannot mult {l} and {r} together"),
        }
    }

    pub fn gt(self, right: Self) -> Self {
        match (self, right) {
            (Int(a), Int(b)) => Bool(a > b),
            (Float(a), Float(b)) => Bool(a > b),
            (l, r) => panic!("cannot mult {l} and {r} together"),
        }
    }

    pub fn ge(self, right: Self) -> Self {
        match (self, right) {
            (Int(a), Int(b)) => Bool(a >= b),
            (Float(a), Float(b)) => Bool(a >= b),
            (l, r) => panic!("cannot mult {l} and {r} together"),
        }
    }

    pub fn lt(self, right: Self) -> Self {
        match (self, right) {
            (Int(a), Int(b)) => Bool(a < b),
            (Float(a), Float(b)) => Bool(a < b),
            (l, r) => panic!("cannot mult {l} and {r} together"),
        }
    }

    pub fn le(self, right: Self) -> Self {
        match (self, right) {
            (Int(a), Int(b)) => Bool(a <= b),
            (Float(a), Float(b)) => Bool(a <= b),
            (l, r) => panic!("cannot mult {l} and {r} together"),
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
            Null => write!(f, "<NULL>"),
            Function(func) => write!(f, "<paraan '{}'>", func.name),
            Str(s) => write!(f, "<interned_string id:{s}>"),
        }
    }
}
