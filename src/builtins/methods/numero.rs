use crate::{
    builtins,
    tol::diagnostic::{Label, runtime::RuntimeError},
    vm::{VM, value::Value},
};

fn expect_numeric_argument(vm: &mut VM, args: &[Value]) -> Result<f64, Box<RuntimeError>> {
    match args.first().unwrap() {
        Value::Int(int) => Ok(*int as f64),
        Value::Float(float) => Ok(*float),
        _ => {
            let err = vm.new_runtime_error("umaasa ako ng numero na argumento", vm.current_ip());
            Err(Box::new(err))
        }
    }
}

pub fn abs(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 1)?;
    match args.first().unwrap() {
        Value::Int(int) => {
            if *int == i64::MIN {
                let err = vm.new_runtime_error(
                    &format!("hindi pwedeng tawagin ang `abs` sa {}", int),
                    vm.current_ip(),
                );
                return Err(Box::new(err));
            }

            Ok(Value::Int(int.abs()))
        }
        Value::Float(float) => Ok(Value::Float(float.abs())),
        _ => Err(Box::new(vm.new_runtime_error(
            "umaasa ako ng numero na argumento",
            vm.current_ip(),
        ))),
    }
}

pub fn bilang_lutang(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 1)?;
    Ok(Value::Float(expect_numeric_argument(vm, args)?))
}

pub fn bilang_string(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 1)?;
    let numeric = expect_numeric_argument(vm, args)?;

    let id = vm.intern_string(&numeric.to_string());
    Ok(Value::Str(id))
}

pub fn bilang_karakter(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 1)?;
    let numeric = expect_numeric_argument(vm, args)?;

    if numeric.fract() != 0.0 {
        return Err(Box::new(vm.new_runtime_error(
            "umaasa ang `bilangKarakter()` ng buong numero na argumento",
            vm.current_ip(),
        )));
    }

    if !(0.0..=255.0).contains(&numeric) {
        return Err(Box::new(vm.new_runtime_error(
            "umaasa ang `bilangKarakter()` ng numerong nasa pagitan ng 0 at 255",
            vm.current_ip(),
        )));
    }

    let id = vm.intern_string(&((numeric as u8) as char).to_string());
    Ok(Value::Str(id))
}
