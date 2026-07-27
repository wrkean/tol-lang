use std::{cell::RefCell, rc::Rc};

use crate::{
    builtin::expected_args_count,
    tol::diagnostic::runtime::RuntimeError,
    vm::{VM, list::List, value::Value},
};

fn expect_string_argument(vm: &mut VM, args: &[Value]) -> Result<Rc<str>, Box<RuntimeError>> {
    let Value::Str(id) = args.first().unwrap() else {
        let err = vm.new_runtime_error("umaasa ako ng lista na argumento", vm.current_ip());
        return Err(Box::new(err));
    };
    let string = vm.get_interned_string(*id);

    Ok(string)
}

pub fn haba(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    expected_args_count(vm, args.len(), 1)?;
    let string = expect_string_argument(vm, args)?;

    Ok(Value::Int(string.len() as i64))
}
