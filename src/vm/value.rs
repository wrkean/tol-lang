#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Null,
}

use std::fmt;

use Value::*;
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
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Int(x) => write!(f, "{x}"),
            Float(x) => write!(f, "{x}"),
            Null => write!(f, "<NULL>"),
        }
    }
}
