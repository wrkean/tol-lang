use std::{cell::RefCell, rc::Rc};

use crate::{
    builtins,
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
    builtins::expected_args_count(vm, args.len(), 1)?;
    let string = expect_string_argument(vm, args)?;

    Ok(Value::Int(string.len() as i64))
}

pub fn bilang_numero(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 1)?;
    let string = expect_string_argument(vm, args)?;

    match string.parse::<i64>() {
        Ok(n) => Ok(Value::Int(n)),
        Err(e) => Err(Box::new(vm.new_runtime_error(
            &format!("may naganap na error sa pag parse ng string bilang numero: {e}"),
            vm.current_ip(),
        ))),
    }
}

pub fn titik(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 1)?;
    let string = expect_string_argument(vm, args)?;

    if string.len() > 1 {
        return Err(Box::new(vm.new_runtime_error(
            &format!(
                "maaaring isang letra lamang ang pwede na itawag dito (nagbigay ka ng {} na letra)",
                string.len()
            ),
            vm.current_ip(),
        )));
    }

    let char_ = string.chars().next().unwrap();
    Ok(Value::Int(char_ as i64))
}
