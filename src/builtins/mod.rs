use crate::{tol::diagnostic::runtime::RuntimeError, vm::VM};

pub mod methods;
pub mod native_functions;

pub fn expected_args_count(
    vm: &VM,
    args_count: usize,
    expected_count: usize,
) -> Result<(), Box<RuntimeError>> {
    if args_count != expected_count {
        let err = vm.new_runtime_error(
            &format!(
                "umaasa ako ng {} na argumento, ngunit {} ang naibigay",
                expected_count, args_count
            ),
            vm.current_ip(),
        );

        return Err(Box::new(err));
    }

    Ok(())
}
