//! Module composed of functions that is responsible for orchestrating the entire compilation
//! process

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    Args,
    global_ctx::{GlobalContext, Module, ModuleCompileState, ModuleId},
    parse::{Parser, lexer::Lexer},
};

/// Compiles the entry point derived from the initialized CLI arguments.
pub fn compile_entry_point(ctx: &mut GlobalContext) {
    let main_module = module_from_path(ctx.entry_point().clone());
    let id = ctx.register_module(main_module);

    compile_module(id, ctx);
}

/// Compiles the given module by module id
pub fn compile_module(module_id: ModuleId, ctx: &mut GlobalContext) {
    let module = ctx.module_by_id_mut(module_id);
    module.set_compile_state(ModuleCompileState::Compiling);
    parse_module(module_id, ctx);
}

fn parse_module(module_id: ModuleId, ctx: &mut GlobalContext) {
    let module = ctx.module_by_id(module_id);

    let tokens = Lexer::new(module.source()).lex();
    let expr = Parser::new(tokens, ctx, module_id).parse();
    println!("{expr}");
}

fn module_from_path(path: impl Into<PathBuf> + AsRef<Path>) -> Module {
    let path = path.into();
    let name = path.file_stem().unwrap().to_str().unwrap().to_string();
    let source = fs::read_to_string(&path).unwrap();

    Module::new(path, name, source)
}
