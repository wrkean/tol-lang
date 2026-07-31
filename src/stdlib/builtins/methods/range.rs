use std::{cell::RefCell, rc::Rc};

use crate::{
    stdlib::builtins,
    tol::diagnostic::runtime::RuntimeError,
    vm::{VM, iterators::RangeIterator, list::List, range::Range, value::Value},
};

fn expect_range_argument(vm: &mut VM, args: &[Value]) -> Result<Rc<Range>, Box<RuntimeError>> {
    let Value::Range(r) = args.first().unwrap() else {
        let err = vm.new_runtime_error("umaasa ako ng lista na argumento", vm.current_ip());
        return Err(Box::new(err));
    };

    Ok(r.clone())
}

pub fn hakbang(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 2)?;
    let range = expect_range_argument(vm, args)?;
    let step = match &args[1] {
        Value::Int(step) => *step,
        _ => {
            let err = vm.new_runtime_error(
                "umaasa ang `hakbang()` ng numero na argumento: `(1..2).hakbang(2)",
                vm.current_ip(),
            );

            return Err(Box::new(err));
        }
    };

    let range = Range {
        start: range.start,
        end: range.end,
        step,
        inclusive: range.inclusive,
    };

    Ok(Value::Range(Rc::new(range)))
}

pub fn __maging_iter__(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 1)?;
    let range = expect_range_argument(vm, args)?;

    let range_iter = RangeIterator::new(&range);

    Ok(Value::Iterator(Rc::new(RefCell::new(range_iter))))
}
