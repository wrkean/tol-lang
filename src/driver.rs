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
    codegen::bytecode_compiler::BytecodeCompiler,
    global_ctx::GlobalContext,
    module::{Module, ModuleCompileState, ModuleId},
    parse::{Parser, lexer::Lexer},
    prelude::DiagResult,
    tol::diagnostic::{TolDiagnostic, miette_diagnostic::MietteDiagnostic},
    vm::VM,
};

/// Compiles the entry point derived from the initialized CLI arguments.
pub fn compile_entry_point(ctx: &mut GlobalContext) -> Result<(), miette::Report> {
    let main_module = module_from_path(ctx.entry_point().clone())?;
    let id = ctx.register_module(main_module);

    compile_module(id, ctx).map_err(|err| miette::Report::new(MietteDiagnostic::from(*err)))?;

    Ok(())
}

/// Compiles the given module by module id
pub fn compile_module(module_id: ModuleId, ctx: &mut GlobalContext) -> DiagResult<()> {
    let module = ctx.module_by_id_mut(module_id);
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

    module.set_compile_state(ModuleCompileState::Compiling);
    parse_module(module_id, ctx);
    analyze_module(module_id, ctx);

    let module = ctx.module_by_id_mut(module_id);

    // Stop compilation
    if module.has_an_error() {
        module.report_diagnostics();
        return Ok(());
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

fn module_from_path(path: impl Into<PathBuf> + AsRef<Path>) -> Result<Module, miette::Report> {
    let path = path.into();
    if !path.exists() {
        return Err(miette!(
            severity = miette::Severity::Error,
            help = "tiyakin na nag-eexist ang path",
            "hindi nag-eexist ang file '{}'",
            path.to_str().unwrap(),
        ));
    }
    let path = path.canonicalize().map_err(|e| {
        miette!(
            severity = miette::Severity::Error,
            "nabigong i-canonicalize ang path '{}': {e}",
            path.to_str().unwrap(),
        )
    })?;
    let name = path.file_stem().unwrap().to_str().unwrap().to_string();
    let source = fs::read_to_string(&path).unwrap();

    Ok(Module::new(path, name, source))
}
