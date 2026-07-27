use crate::{
    tol::diagnostic::runtime::RuntimeError,
    vm::{self, VM, value::Value},
};

pub fn dagdag(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    if args.len() != 2 {
        let err = vm.new_runtime_error(
            &format!(
                "umaasa ako ng 2 na argumento, ngunit {} ang naibigay",
                args.len()
            ),
            vm.current_ip(),
        );

        return Err(Box::new(err));
    }

    let Value::List(list) = args.first().unwrap() else {
        let err = vm.new_runtime_error("umaasa ako ng lista na argumento", vm.current_ip());
        return Err(Box::new(err));
    };

    list.borrow_mut().elements.push(args[1].clone());

    Ok(Value::Null)
}
