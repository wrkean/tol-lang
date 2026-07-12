use std::{
    cell::{Ref, RefCell},
    collections::{HashMap, hash_map::Entry},
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use crate::Args;

pub struct Compiler {
    entry_point: PathBuf,
    modules: HashMap<PathBuf, Rc<RefCell<Module>>>,
}

impl Compiler {
    pub fn new(cli_args: Args) -> Self {
        Self {
            entry_point: cli_args.input,
            modules: HashMap::new(),
        }
    }

    pub fn compile_entry_point(&mut self) {
        let main_module = self.module_from_path(self.entry_point.clone());
    }

    fn module_from_path(&mut self, path: impl Into<PathBuf> + AsRef<Path>) -> Module {
        let path = path.into();
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let source = fs::read_to_string(&path).unwrap();

        Module::new(path, name, source)
    }

    fn compile_module(&mut self, module: Module) -> Rc<RefCell<Module>> {
        let path = module.path.clone();

        if !self.modules.contains_key(&path) {
            self.modules
                .insert(path.clone(), Rc::new(RefCell::new(module)));
        }

        self.modules.get(&path).unwrap().clone()
    }
}

pub struct Module {
    path: PathBuf,
    name: String,
    source: Arc<str>,
    compile_state: ModuleCompileState,
}

impl Module {
    pub fn new(path: PathBuf, name: String, source: String) -> Self {
        Self {
            path,
            name,
            source: Arc::from(source),
            compile_state: ModuleCompileState::Initialized,
        }
    }
}

pub enum ModuleCompileState {
    Initialized,
    Compiling,
    Compiled,
}
