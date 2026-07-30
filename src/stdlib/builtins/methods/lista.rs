use std::{cell::RefCell, rc::Rc};

use crate::{
    stdlib::builtins,
    tol::diagnostic::runtime::RuntimeError,
    vm::{self, VM, list::List, value::Value},
};

fn expect_list_argument(
    vm: &mut VM,
    args: &[Value],
) -> Result<Rc<RefCell<List>>, Box<RuntimeError>> {
    let Value::List(list) = args.first().unwrap() else {
        let err = vm.new_runtime_error("umaasa ako ng lista na argumento", vm.current_ip());
        return Err(Box::new(err));
    };

    Ok(list.clone())
}

pub fn dagdag(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 2)?;
    let list = expect_list_argument(vm, args)?;

    list.borrow_mut().elements.push(args[1].clone());

    Ok(Value::Null)
}

pub fn haba(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 1)?;
    let list = expect_list_argument(vm, args)?;

    Ok(Value::Int(list.borrow().elements.len() as i64))
}
