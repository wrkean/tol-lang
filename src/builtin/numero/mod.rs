use crate::{
    tol::diagnostic::{Label, runtime::RuntimeError},
    vm::{VM, value::Value},
};

pub fn abs(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    if args.len() != 1 {
        let err = vm.new_runtime_error(
            &format!(
                "umaasa ako ng 1 na argumento, ngunit {} ang naibigay",
                args.len()
            ),
            vm.current_ip(),
        );

        return Err(Box::new(err));
    }

    let Value::Int(int) = args.first().unwrap() else {
        unreachable!()
    };

    print!("{}", i64::MIN);
    if *int == i64::MIN {
        let err = vm.new_runtime_error(
            &format!("hindi pwedeng tawagin ang `abs` sa {}", int),
            vm.current_ip(),
        );
        return Err(Box::new(err));
    }

    Ok(Value::Int(int.abs()))
}

pub fn maging_string(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    if args.len() != 1 {
        let err = vm.new_runtime_error(
            &format!(
                "umaasa ako ng 1 na argumento, ngunit {} ang naibigay",
                args.len()
            ),
            vm.current_ip(),
        );

        return Err(Box::new(err));
    }

    let Value::Int(int) = args.first().unwrap() else {
        let err = vm.new_runtime_error("umaasa ako ng numero dito", vm.current_ip());
        return Err(Box::new(err));
    };

    let id = vm.intern_string(&int.to_string());
    Ok(Value::Str(id))
}
