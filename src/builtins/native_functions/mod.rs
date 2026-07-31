use crate::{
    builtins,
    tol::diagnostic::{Label, runtime::RuntimeError},
    vm::{VM, value::Value},
};

pub mod io;
pub mod math;

pub type NativeFn = fn(&mut VM, &[Value]) -> Result<Value, Box<RuntimeError>>;

#[derive(Debug)]
pub struct NativeFunction {
    pub name: String,
    pub arity: Option<usize>,
    pub func: NativeFn,
}
