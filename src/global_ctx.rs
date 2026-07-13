use std::{
    cell::{Ref, RefCell},
    collections::{HashMap, hash_map::Entry},
    fs, mem,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use crate::{
    Args,
    module::{Module, ModuleId},
    parse::{
        Parser,
        ast::{Ast, stmt::Stmt},
        lexer::Lexer,
    },
    tol::diagnostic::{Severity, TolDiagnostic, miette_diagnostic::MietteDiagnostic},
};

/// Stores all the information of the whole compilation pipeline
pub struct GlobalContext {
    // Entry point derived from CLI arguments
    // TODO: Make this optional later when we support REPLs
    entry_point: PathBuf,

    // Module table, accessed via module id
    modules: Vec<Module>,

    // Acts as a cache for loaded modules
    module_registry: HashMap<PathBuf, ModuleId>,
}

impl GlobalContext {
    /// Creates a new global context with the arguments
    pub fn new(cli_args: Args) -> Self {
        Self {
            entry_point: cli_args.input,
            modules: Vec::new(),
            module_registry: HashMap::new(),
        }
    }

    /// Registers the module into the module registry.
    ///
    /// If the module already exists in the registry, it returns the module id defined in that registry.
    /// Otherwise, it pushes the module into the module table, registers it, and returns the module
    /// id pointing to it
    pub fn register_module(&mut self, module: Module) -> ModuleId {
        let path = module.path();

        if !self.module_registry.contains_key(path) {
            let id = self.modules.len();
            self.module_registry.insert(path.clone(), id);

            self.modules.push(module);
            return id;
        }

        *self.module_registry.get(path).unwrap()
    }

    /// Retrieves a reference to a module at the given index
    pub fn module_by_id(&self, index: usize) -> &Module {
        &self.modules[index]
    }

    /// Retrieves a mutable reference to a module at the given index
    pub fn module_by_id_mut(&mut self, index: usize) -> &mut Module {
        &mut self.modules[index]
    }

    pub fn entry_point(&self) -> &PathBuf {
        &self.entry_point
    }

    pub fn modules(&self) -> &[Module] {
        &self.modules
    }
}
