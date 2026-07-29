use std::io::{self, Write};

use crate::{
    analyze::ResolvedVar,
    builtin,
    tol::diagnostic::{Label, runtime::RuntimeError},
    vm::{VM, value::Value},
};

pub type NativeFn = fn(&mut VM, &[Value]) -> Result<Value, Box<RuntimeError>>;

#[derive(Debug)]
pub struct NativeFunction {
    pub name: String,
    pub arity: Option<usize>,
    pub func: NativeFn,
}

pub fn native_input(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtin::expected_args_count(vm, args.len(), 1);
    match args.first() {
        Some(arg) => {
            vm.print_value(arg);
            io::stdout().flush();

            let mut input = String::new();

            if let Err(e) = io::stdin().read_line(&mut input) {
                let current_module = vm.current_module();
                return Err(Box::new(RuntimeError::new(
                    current_module.source_arc(),
                    current_module.filename(),
                    e.to_string(),
                    Label::new(0..0),
                )));
            }

            let id = vm.intern_string(&input);

            Ok(Value::Str(id))
        }
        None => {
            let current_module = vm.current_module();
            let err = RuntimeError::new(
                current_module.source_arc(),
                current_module.filename(),
                "ang input() ay umaasa ng kahit isang argumento",
                Label::new(0..0),
            );

            Err(Box::new(err))
        }
    }
}

pub fn native_alis(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtin::expected_args_count(vm, args.len(), 1)?;

    match args.first() {
        Some(arg) => {
            let current_module = vm.current_module();
            let Value::Int(exit_code) = arg else {
                return Err(Box::new(RuntimeError::new(
                    current_module.source_arc(),
                    current_module.filename(),
                    "umaasa ng numero na argumento",
                    Label::new(0..0),
                )));
            };

            std::process::exit((*exit_code).clamp(0, 255) as i32);

            Ok(Value::Null)
        }
        None => {
            let current_module = vm.current_module();
            let err = RuntimeError::new(
                current_module.source_arc(),
                current_module.filename(),
                "ang alis() ay umaasa ng kahit isang argumento",
                Label::new(0..0),
            );

            Err(Box::new(err))
        }
    }
}

pub fn native_print(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    let template = match args.first() {
        Some(Value::Str(id)) => vm.get_interned_string(*id),
        Some(_) => {
            return Err(Box::new(vm.new_runtime_error(
                "ang print() ay umaasa ng string bilang pangunahing argumento",
                vm.current_ip(),
            )));
        }
        None => {
            return Err(Box::new(vm.new_runtime_error(
                "ang print() ay umaasa kahit isang argumento",
                vm.current_ip(),
            )));
        }
    };

    let sub_args = &args[1..];
    let output = format_template(vm, &template, sub_args)?;

    print!("{}", output);
    io::stdout().flush().map_err(|e| {
        Box::new(vm.new_runtime_error(
            &format!("nabigong i-flush ang stdout: {}", e),
            vm.current_ip(),
        ))
    })?;

    Ok(Value::Null)
}

pub fn native_println(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    let template = match args.first() {
        Some(Value::Str(id)) => vm.get_interned_string(*id),
        Some(_) => {
            return Err(Box::new(vm.new_runtime_error(
                "ang print() ay umaasa ng string bilang pangunahing argumento",
                vm.current_ip(),
            )));
        }
        None => {
            return Err(Box::new(vm.new_runtime_error(
                "ang print() ay umaasa kahit isang argumento",
                vm.current_ip(),
            )));
        }
    };

    let sub_args = &args[1..];
    let output = format_template(vm, &template, sub_args)?;

    println!("{}", output);

    Ok(Value::Null)
}

/// Formats a template string by substituting each '{}' placeholder
/// with the corresponding argument, in order.
/// Shared by both `native_print` and `native_println`.
fn format_template(
    vm: &VM,
    template: &str,
    sub_args: &[Value],
) -> Result<String, Box<RuntimeError>> {
    let mut result = String::with_capacity(template.len());
    let mut arg_index = 0;
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'}') {
            chars.next(); // consume closing '}'

            let value = sub_args.get(arg_index).ok_or_else(|| {
                Box::new(vm.new_runtime_error(&format!(
                        "ang format ay umaasa ng mga argumento: hindi mahanap na value para sa placeholder ika-{}",
                    arg_index
                ), vm.current_ip()))
            })?;

            result.push_str(&value.to_string());
            arg_index += 1;
        } else {
            result.push(c);
        }
    }

    Ok(result)
}
