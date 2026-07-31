//! Module composed of functions that is responsible for orchestrating the entire compilation
//! process

use std::{
    fs,
    path::{Path, PathBuf},
};

use miette::miette;

use crate::{
    Args,
    analyze::Analyzer,
    builtins::{self, native_functions},
    codegen::bytecode_compiler::BytecodeCompiler,
    global_ctx::GlobalContext,
    module::{Module, ModuleCompileState, ModuleId},
    parse::{Parser, lexer::Lexer},
    prelude::DiagResult,
    tol::diagnostic::{TolDiagnostic, miette_diagnostic::MietteDiagnostic},
    vm::VM,
};

/// Compiles the entry point derived from the initialized CLI arguments.
pub fn compile_entry_point(ctx: &mut GlobalContext) -> DiagResult<()> {
    let main_module = module_from_path(ctx.entry_point().clone())?;
    let id = ctx.register_module(main_module);

    compile_module(id, ctx, true)?;

    Ok(())
}

/// Compiles the given module by module id
pub fn compile_module(
    module_id: ModuleId,
    ctx: &mut GlobalContext,
    do_attach_stdlib: bool,
) -> DiagResult<()> {
    let module = ctx.module_by_id_mut(module_id);
    if do_attach_stdlib {
        attach_stdlib(module_id, ctx)?;
    }

    let module = ctx.module_by_id(module_id);
    match module.compile_state() {
        ModuleCompileState::Compiling => return Err(Box::new(TolDiagnostic::err(
            module.source_arc(),
            module.filename(),
            "kasalukuyang kino-compile ang module na ito",
        ).help("maaaring nangyari ito dahil ang module na kinuha mo ay kinuha rin ang module na kumuha nito"))),

        // Is already compiled, no need to compile again
        ModuleCompileState::Compiled => return Ok(()),

        // Do nothing, proceed
        ModuleCompileState::Initialized => {}
    }

    let module = ctx.module_by_id_mut(module_id);
    module.set_compile_state(ModuleCompileState::Compiling);
    parse_module(module_id, ctx);
    analyze_module(module_id, ctx);

    let module = ctx.module_by_id_mut(module_id);
    // Stop compilation
    if module.has_an_error() {
        module.report_diagnostics();
        let diagnostic = TolDiagnostic::err(
            module.source_arc(),
            module.filename(),
            format!(
                "hindi ma-itakbo ang `{}` dahil sa mga error",
                module.filename()
            ),
        );
        return Err(Box::new(diagnostic));
    }

    let mut compiler = BytecodeCompiler::new(ctx, module_id);
    let chunk = compiler.compile();

    let module = ctx.module_by_id_mut(module_id);
    module.set_chunk(chunk);
    module.set_compile_state(ModuleCompileState::Compiled);
    module.report_diagnostics();

    Ok(())
}

/// Runs the whole thing
pub fn run(entry_module: ModuleId, ctx: GlobalContext) {
    let mut vm = ctx.into_vm();
    vm.run();
}

fn parse_module(module_id: ModuleId, ctx: &mut GlobalContext) {
    let module = ctx.module_by_id(module_id);
    let source = module.source_arc();

    let tokens = Lexer::new(&source, ctx, module_id).lex();
    Parser::new(tokens, ctx, module_id).parse();
}

fn analyze_module(module_id: ModuleId, ctx: &mut GlobalContext) {
    let mut analyzer = Analyzer::new(ctx, module_id);
    analyzer.analyze();
}

pub fn module_from_path(path: impl Into<PathBuf> + AsRef<Path>) -> DiagResult<Module> {
    let path = path.into();
    if !path.exists() {
        let diagnostic = TolDiagnostic::err_no_source(format!(
            "hindi nag-eexist ang file '{}'",
            path.to_str().unwrap_or("<hindi_ma_parse_na_pangalan>")
        ));
        return Err(Box::new(diagnostic));
    }
    let path = path
        .canonicalize()
        .map_err(|e| TolDiagnostic::err_no_source(e.to_string()))?;
    let name = path.file_stem().unwrap().to_str().unwrap().to_string();
    let source = fs::read_to_string(&path).unwrap();

    Ok(Module::new(path, name, source))
}

pub fn attach_stdlib(target_module_id: ModuleId, ctx: &mut GlobalContext) -> DiagResult<()> {
    let target_module = ctx.module_by_id(target_module_id);
    let source_arc = target_module.source_arc();
    let filename = target_module.filename();
    let map_err_to_diagnostic = |diagnostic: Box<TolDiagnostic>| {
        diagnostic
            .source(source_arc.clone())
            .filename(filename.clone())
    };

    attach_std_io(target_module_id, ctx)
        .map_err(map_err_to_diagnostic)
        .map_err(|err| err.message("nabigong i-compile ang io na module, na-iset mo ba ang environment variable na TOL_STDLIB?"))?;
    attach_std_math(target_module_id, ctx)
        .map_err(map_err_to_diagnostic)
        .map_err(|err| err.message("nabigong i-compile ang math na module, na-iset mo ba ang environment variable na TOL_STDLIB?"))?;

    Ok(())
}

fn attach_std_io(target_module_id: ModuleId, ctx: &mut GlobalContext) -> DiagResult<()> {
    let stdlib_path = ctx.stdlib_path();
    let io_module = module_from_path(stdlib_path.join("io.tol"))?;
    let io_module_id = ctx.register_module(io_module);
    native_functions::io::initialize_io_module(io_module_id, ctx);
    compile_module(io_module_id, ctx, false)?;
    ctx.module_by_id_mut(target_module_id)
        .add_dependency(io_module_id);

    Ok(())
}

fn attach_std_math(target_module_id: ModuleId, ctx: &mut GlobalContext) -> DiagResult<()> {
    let stdlib_path = ctx.stdlib_path();
    let math_module = module_from_path(stdlib_path.join("math.tol"))?;
    let math_module_id = ctx.register_module(math_module);
    native_functions::math::initialize_math_module(math_module_id, ctx);
    compile_module(math_module_id, ctx, false)?;
    ctx.module_by_id_mut(target_module_id)
        .add_dependency(math_module_id);

    Ok(())
}
