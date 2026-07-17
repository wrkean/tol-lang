//! Module composed of functions that is responsible for orchestrating the entire compilation
//! process.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    analyze::Analyzer,
    codegen::bytecode_compiler::BytecodeCompiler,
    global_ctx::GlobalContext,
    module::{Module, ModuleCompileState, ModuleId},
    parse::{Parser, lexer::Lexer},
    tol::diagnostic::miette_diagnostic::MietteDiagnostic,
    vm::VM,
};

#[derive(Default, Debug, Clone)]
pub struct RunResult {
    pub output: String,
    pub diagnostics: Vec<String>,
    pub success: bool,
}

/// Runs a Tol source file and returns its printed output and diagnostics.
pub fn run_file(path: impl Into<PathBuf> + AsRef<Path>) -> RunResult {
    let path = path.into();
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(err) => {
            return RunResult {
                diagnostics: vec![format!("failed to read `{}`: {}", path.display(), err)],
                ..RunResult::default()
            };
        }
    };

    run_source(source, path.display().to_string())
}

/// Runs Tol source code and returns its printed output and diagnostics.
pub fn run_source(source: impl Into<String>, filename: impl Into<String>) -> RunResult {
    let filename = filename.into();
    let path = PathBuf::from(&filename);
    let module = module_from_source(path, source.into());
    let mut ctx = GlobalContext::new(filename);
    let id = ctx.register_module(module);

    run_module(id, &mut ctx)
}

/// Compiles the given module by module id.
fn run_module(module_id: ModuleId, ctx: &mut GlobalContext) -> RunResult {
    let module = ctx.module_by_id_mut(module_id);
    module.set_compile_state(ModuleCompileState::Compiling);
    parse_module(module_id, ctx);
    analyze_module(module_id, ctx);

    let diagnostics = {
        let module = ctx.module_by_id_mut(module_id);
        if module.has_an_error() {
            let diagnostics = module.drain_diagnostics_as_strings();
            return RunResult {
                diagnostics,
                ..RunResult::default()
            };
        }
        Vec::new()
    };

    let mut compiler = BytecodeCompiler::new(ctx, module_id);
    let chunk = compiler.compile();

    let mut vm = VM::new(chunk, ctx, module_id);
    let run_result = vm.run();
    let output = vm.take_output().join("\n");

    let mut result = RunResult {
        output,
        diagnostics,
        success: run_result.is_ok(),
    };

    if let Err(e) = run_result {
        result.diagnostics.push(format!(
            "{:?}",
            miette::Report::new(MietteDiagnostic::from(*e))
        ));
    }

    let module = ctx.module_by_id_mut(module_id);
    result
        .diagnostics
        .extend(module.drain_diagnostics_as_strings());

    result
}

fn parse_module(module_id: ModuleId, ctx: &mut GlobalContext) {
    let module = ctx.module_by_id(module_id);
    let source = module.source_arc();

    let tokens = Lexer::new(&source, ctx, module_id).lex();
    for tok in tokens.iter() {
        println!("{:?}", tok.kind());
    }
    Parser::new(tokens, ctx, module_id).parse();
}

fn analyze_module(module_id: ModuleId, ctx: &mut GlobalContext) {
    let mut analyzer = Analyzer::new(ctx, module_id);
    analyzer.analyze();
}

fn module_from_source(path: impl Into<PathBuf> + AsRef<Path>, source: String) -> Module {
    let path = path.into();
    let name = path.file_stem().unwrap().to_str().unwrap().to_string();
    Module::new(path, name, source)
}
