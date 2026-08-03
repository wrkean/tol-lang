use rand::RngExt;

use crate::{
    builtins,
    global_ctx::GlobalContext,
    module::ModuleId,
    natives,
    tol::diagnostic::runtime::RuntimeError,
    vm::{VM, value::Value},
};

pub fn initialize_math_module(math_module_id: ModuleId, ctx: &mut GlobalContext) {
    let module = ctx.module_by_id_mut(math_module_id);
    natives!(module,
        "abs" => Some(1) => native_abs,
        "sqrt" => Some(1) => native_sqrt,
        "random" => Some(0) => native_random
    )
}

pub fn native_abs(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 1)?;
    match args.first() {
        Some(Value::Int(x)) => Ok(Value::Int(x.abs())),
        Some(Value::Float(x)) => Ok(Value::Float(x.abs())),
        _ => {
            let err = vm.new_runtime_error("maaaring numero (1, 2, 3, ...) o lutang (1.1, 1.2, 1.3, ...) lamang ang pwedeng argumento dito ng `abs()`", vm.current_ip());
            Err(Box::new(err))
        }
    }
}

pub fn native_sqrt(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 1)?;
    match args.first() {
        Some(Value::Int(x)) => Ok(Value::Float((*x as f64).sqrt())),
        Some(Value::Float(x)) => Ok(Value::Float(x.sqrt())),
        _ => {
            let err = vm.new_runtime_error("maaaring numero (1, 2, 3, ...) o lutang (1.1, 1.2, 1.3, ...) lamang ang pwedeng argumento dito ng `abs()`", vm.current_ip());
            Err(Box::new(err))
        }
    }
}

pub fn native_random(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 0)?;
    let random = rand::rng().random::<f64>();

    Ok(Value::Float(random))
}
