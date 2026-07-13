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
    parse::{
        Parser,
        ast::{Ast, stmt::Stmt},
        lexer::Lexer,
    },
    tol::diagnostic::{Severity, TolDiagnostic, miette_diagnostic::MietteDiagnostic},
};

pub type ModuleId = usize;

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
        let path = &module.path;

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

/// Holds all the information of a file.
///
/// In tol, each file is a module
pub struct Module {
    // This module's path
    path: PathBuf,

    // This module's name, derived from the path
    name: String,

    // The atomically referenced counted source
    source: Arc<str>,

    // The compilation state of this module
    compile_state: ModuleCompileState,

    // The Abstract Syntax Tree for this module
    ast: Ast,

    // Diagnostics accumulated during the compilation of this module
    diagnostics: Vec<TolDiagnostic>,

    // True when one of the diagnostic has a severity = Error
    has_an_error: bool,
}

impl Module {
    /// Creates a new module derived from the given arguments
    pub fn new(path: PathBuf, name: String, source: String) -> Self {
        Self {
            path,
            name,
            source: Arc::from(source),
            compile_state: ModuleCompileState::Initialized,
            ast: Ast::new(),
            diagnostics: Vec::new(),
            has_an_error: false,
        }
    }

    /// Get the source
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get an Arc clone of the source
    pub fn source_arc(&self) -> Arc<str> {
        self.source.clone()
    }

    /// Sets the compile state of the module
    pub fn set_compile_state(&mut self, compile_state: ModuleCompileState) {
        self.compile_state = compile_state;
    }

    /// Adds a new statement to the ast
    pub fn add_statement(&mut self, statement: Stmt) {
        self.ast.push(statement);
    }

    /// Adds a new diagnostic to this module
    pub fn add_diagnostic(&mut self, diagnostic: TolDiagnostic) {
        if diagnostic.severity() == &Severity::Error {
            self.has_an_error = true;
        }

        self.diagnostics.push(diagnostic);
    }

    /// The module name + .tol extension
    pub fn filename(&self) -> String {
        self.name.clone() + ".tol"
    }

    /// This module has an error if one of the pushed diagnostic has a severity = Error
    pub fn has_an_error(&self) -> bool {
        self.has_an_error
    }

    /// Reports the diagnostics for this module one by one
    ///
    /// NOTE: This consumes this module's diagnostics. As such, this is only to be called one
    /// per module as the diagnostics are already consumed at the point when this is accessed again
    pub fn report_diagnostics(&mut self) {
        let diagnostics = mem::take(&mut self.diagnostics);
        for diagnostic in diagnostics {
            eprintln!(
                "{:?}",
                miette::Report::new(MietteDiagnostic::from(diagnostic))
            );
        }
    }
}

/// A module's compile state, composed of three states:
///
/// - Initialized: Initial state of the module upon creating it
/// - Compiling: State of the module if it is being currently compiled (being lexed/parsed/analyzed/compiled)
/// - Compiled: State of the module after being compiled, which holds its own bytecode
pub enum ModuleCompileState {
    Initialized,
    Compiling,
    Compiled,
}
