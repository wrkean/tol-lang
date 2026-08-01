use std::io::{self, Write};

use crate::{
    builtins,
    global_ctx::GlobalContext,
    module::ModuleId,
    tol::diagnostic::{Label, runtime::RuntimeError},
    vm::{VM, value::Value},
};

pub fn initialize_io_module(io_module_id: ModuleId, ctx: &mut GlobalContext) {
    let module = ctx.module_by_id_mut(io_module_id);
    module.add_native_fn("input", Some(1), native_input);
    module.add_native_fn("isulat", None, native_print);
    module.add_native_fn("isulatln", None, native_println);
}

pub fn native_input(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 0)?;
    let mut input = String::new();

    if let Err(e) = io::stdin().read_line(&mut input) {
        let current_module = vm.current_module();
        return Err(Box::new(RuntimeError::new(
            current_module.source_arc(),
            current_module.filename(),
            format!("Nabigong kunin ang input: {}", e),
            Label::new(0..0),
        )));
    }

    let input = input.trim_end();
    let id = vm.intern_string(input);

    Ok(Value::Str(id))
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
                        "ang format ay umaasa ng mga argumento: hindi mahanap na value para sa placeholder ika-{} na karakter",
                    arg_index
                ), vm.current_ip()))
            })?;

            result.push_str(&value.to_printed_string(vm));
            arg_index += 1;
        } else {
            result.push(c);
        }
    }

    Ok(result)
}
