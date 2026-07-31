use crate::{
    driver::{self},
    global_ctx::GlobalContext,
    module::ModuleId,
    prelude::DiagResult,
    stdlib::builtins::native_functions,
    tol::diagnostic::TolDiagnostic,
};

pub mod builtins;

// TODO: Replace the hardcoded path

pub fn attach_stdlib(target_module_id: ModuleId, ctx: &mut GlobalContext) -> DiagResult<()> {
    let target_module = ctx.module_by_id(target_module_id);
    let source_arc = target_module.source_arc();
    let filename = target_module.filename();
    let map_err_to_diagnostic = |diagnostic: Box<TolDiagnostic>| {
        diagnostic
            .source(source_arc.clone())
            .filename(filename.clone())
    };

    attach_std_io(target_module_id, ctx).map_err(map_err_to_diagnostic)?;
    attach_std_math(target_module_id, ctx).map_err(map_err_to_diagnostic)?;

    Ok(())
}

fn attach_std_io(target_module_id: ModuleId, ctx: &mut GlobalContext) -> DiagResult<()> {
    let io_module =
        driver::module_from_path("/home/wrkean/Projects/tol-lang/src/stdlib/std/io.tol")?;
    let io_module_id = ctx.register_module(io_module);
    native_functions::io::initialize_io_module(io_module_id, ctx);
    driver::compile_module(io_module_id, ctx, false)?;
    ctx.module_by_id_mut(target_module_id)
        .add_dependency(io_module_id);

    Ok(())
}

fn attach_std_math(target_module_id: ModuleId, ctx: &mut GlobalContext) -> DiagResult<()> {
    let math_module =
        driver::module_from_path("/home/wrkean/Projects/tol-lang/src/stdlib/std/math.tol")?;
    let math_module_id = ctx.register_module(math_module);
    native_functions::math::initialize_math_module(math_module_id, ctx);
    driver::compile_module(math_module_id, ctx, false)?;
    ctx.module_by_id_mut(target_module_id)
        .add_dependency(math_module_id);

    Ok(())
}
