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

pub fn native_alis(_vm: &mut VM, _args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    match _args.first() {
        Some(arg) => {
            let current_module = _vm.current_module();
            let Value::Int(exit_code) = arg else {
                return Err(Box::new(RuntimeError::new(
                    current_module.source_arc(),
                    current_module.filename(),
                    "umaasa ng numero na argumento",
                    Label::new(0..0),
                )));
            };

            std::process::exit((*exit_code).clamp(0, 255) as i32);

            Ok(Value::Null)
        }
        None => {
            let current_module = _vm.current_module();
            let err = RuntimeError::new(
                current_module.source_arc(),
                current_module.filename(),
                "ang alis() ay umaasa ng kahit isang argumento",
                Label::new(0..0),
            );

            Err(Box::new(err))
        }
    }
}
