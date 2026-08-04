use std::{cell::RefCell, rc::Rc};

use crate::{
    builtins,
    tol::diagnostic::runtime::RuntimeError,
    vm::{VM, iterators::RangeIterator, range::Range, value::Value},
};

fn expect_range_argument(vm: &mut VM, args: &[Value]) -> Result<Rc<Range>, Box<RuntimeError>> {
    let Value::Range(r) = args.first().unwrap() else {
        let err = vm.new_runtime_error("umaasa ako ng sakop na argumento", vm.current_ip());
        return Err(Box::new(err));
    };

    Ok(r.clone())
}

pub fn hakbang(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 2)?;
    let range = expect_range_argument(vm, args)?;

    let Some(range) = range.with_step(&args[1]) else {
        let err = vm.new_runtime_error(
            "umaasa ang `hakbang()` ng numero na argumento: `(1..2).hakbang(2)`",
            vm.current_ip(),
        );

        return Err(Box::new(err));
    };

    Ok(Value::Range(Rc::new(range)))
}

pub fn iter(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 1)?;
    let range = expect_range_argument(vm, args)?;

    let range_iter = RangeIterator::new(&range);

    Ok(Value::Iterator(Rc::new(RefCell::new(range_iter))))
}
