use std::{cell::RefCell, rc::Rc};

use crate::{
    builtins,
    tol::diagnostic::runtime::RuntimeError,
    vm::{
        VM,
        iterators::{MapIterator, NativeIterator},
        list::List,
        value::Value,
    },
};

fn expect_iterator_argument(
    vm: &mut VM,
    args: &[Value],
) -> Result<Rc<RefCell<dyn NativeIterator>>, Box<RuntimeError>> {
    let Value::Iterator(iter) = args.first().unwrap() else {
        let err = vm.new_runtime_error("umaasa ako ng iterator na argumento", vm.current_ip());
        return Err(Box::new(err));
    };

    Ok(iter.clone())
}

pub fn map(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 2)?;

    let iter = expect_iterator_argument(vm, args)?;
    let mapper = args[1].clone();
    let map_iter = MapIterator::new(iter.clone(), mapper);

    Ok(Value::Iterator(Rc::new(RefCell::new(map_iter))))
}

pub fn ipunin_sa_lista(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 1)?;

    let iter = expect_iterator_argument(vm, args)?;

    let mut elements = Vec::new();
    while let Some(val) = iter.borrow_mut().next(vm).clone() {
        elements.push(val);
    }

    Ok(Value::List(Rc::new(RefCell::new(List { elements }))))
}
