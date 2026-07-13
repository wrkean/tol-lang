use std::{
    cell::{Ref, RefCell},
    collections::{HashMap, hash_map::Entry},
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use crate::{Args, parse::lexer::Lexer};

pub type ModuleId = usize;

/// Handles the compilation of all the modules. Handles the entire compilation pipeline
pub struct Compiler {
    // Entry point derived from CLI arguments
    // TODO: Make this optional later when we support REPLs
    entry_point: PathBuf,

    // Module table, accessed via module id
    modules: Vec<Module>,

    // Acts as a cache for loaded modules
    module_registry: HashMap<PathBuf, ModuleId>,
}

impl Compiler {
    /// Creates a new compiler with the arguments
    pub fn new(cli_args: Args) -> Self {
        Self {
            entry_point: cli_args.input,
            modules: Vec::new(),
            module_registry: HashMap::new(),
        }
    }

    /// Compiles the entry point derived from the initialized CLI arguments.
    pub fn compile_entry_point(&mut self) {
        let main_module = self.module_from_path(self.entry_point.clone());
        let id = self.register_module(main_module);

        self.compile_module(id);
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

    fn compile_module(&mut self, module_id: ModuleId) {
        let module = &mut self.modules[module_id];
        module.set_compile_state(ModuleCompileState::Compiling);
        self.parse_module(module_id);
    }

    fn parse_module(&mut self, module_id: ModuleId) {
        let module = &self.modules[module_id];

        let tokens = Lexer::new(module.source()).lex();
    }

    fn module_from_path(&mut self, path: impl Into<PathBuf> + AsRef<Path>) -> Module {
        let path = path.into();
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let source = fs::read_to_string(&path).unwrap();

        Module::new(path, name, source)
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
}

impl Module {
    /// Creates a new module derived from the given arguments
    pub fn new(path: PathBuf, name: String, source: String) -> Self {
        Self {
            path,
            name,
            source: Arc::from(source),
            compile_state: ModuleCompileState::Initialized,
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
