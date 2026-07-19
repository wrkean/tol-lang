use std::io::{self, Write};

use crate::{
    tol::diagnostic::{Label, runtime::RuntimeError},
    vm::{VM, value::Value},
};

pub type NativeFn = fn(&mut VM, &[Value]) -> Result<Value, Box<RuntimeError>>;

#[derive(Debug)]
pub struct NativeFunction {
    pub name: String,
    pub arity: usize,
    pub func: NativeFn,
}

pub fn native_input(_vm: &mut VM, _args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    match _args.first() {
        Some(arg) => {
            _vm.print_value(arg);
            io::stdout().flush();

            let mut input = String::new();

            if let Err(e) = io::stdin().read_line(&mut input) {
                let current_module = _vm.current_module();
                return Err(Box::new(RuntimeError::new(
                    current_module.source_arc(),
                    current_module.filename(),
                    e.to_string(),
                    Label::new(0..0),
                )));
            }

            let id = _vm.intern_string(&input);

            Ok(Value::Str(id))
        }
        None => {
            let current_module = _vm.current_module();
            let err = RuntimeError::new(
                current_module.source_arc(),
                current_module.filename(),
                "ang input() ay umaasa ng kahit isang argumento",
                Label::new(0..0),
            );

            Err(Box::new(err))
        }
    }
}
