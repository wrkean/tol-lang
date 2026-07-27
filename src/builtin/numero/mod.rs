use crate::{
    builtin::expected_args_count,
    tol::diagnostic::{Label, runtime::RuntimeError},
    vm::{VM, value::Value},
};

fn expect_int_argument(vm: &mut VM, args: &[Value]) -> Result<i64, Box<RuntimeError>> {
    let Value::Int(int) = args.first().unwrap() else {
        let err = vm.new_runtime_error("umaasa ako ng lista na argumento", vm.current_ip());
        return Err(Box::new(err));
    };

    Ok(*int)
}

pub fn abs(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    expected_args_count(vm, args.len(), 1)?;
    let int = expect_int_argument(vm, args)?;

    if int == i64::MIN {
        let err = vm.new_runtime_error(
            &format!("hindi pwedeng tawagin ang `abs` sa {}", int),
            vm.current_ip(),
        );
        return Err(Box::new(err));
    }

    Ok(Value::Int(int.abs()))
}

pub fn bilang_string(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    expected_args_count(vm, args.len(), 1)?;
    let int = expect_int_argument(vm, args)?;

    let id = vm.intern_string(&int.to_string());
    Ok(Value::Str(id))
}

pub fn bilang_ascii(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    expected_args_count(vm, args.len(), 1)?;
    let int = expect_int_argument(vm, args)?;

    let id = vm.intern_string(&(int as u8 as char).to_string());
    Ok(Value::Str(id))
}
