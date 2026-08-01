use std::{collections::HashMap, rc::Rc};

use crate::{
    builtins::{
        self,
        native_functions::{NativeFn, NativeFunction},
    },
    global_ctx::GlobalContext,
    module::ModuleId,
    tol::diagnostic::runtime::RuntimeError,
    vm::{VM, class::ClassDef, value::Value},
};

pub fn initialize_uri_module(target_module_id: ModuleId, ctx: &mut GlobalContext) {
    let module = ctx.module_by_id_mut(target_module_id);
    let lista_type = initialize_lista_type();
    module.new_global("Lista", lista_type);
}

fn initialize_lista_type() -> Value {
    let mut methods = HashMap::new();

    use builtins::methods::lista::*;
    methods.insert("dagdag".into(), new_native("dagdag", Some(2), dagdag));
    methods.insert("haba".into(), new_native("haba", Some(1), haba));
    methods.insert("bago".into(), new_native("bago", Some(0), bago));
    methods.insert(
        "__maging_iter__".into(),
        new_native("__maging_iter__", Some(1), __maging_iter__),
    );

    let class_def = ClassDef::new("Lista".to_string(), methods);
    Value::ClassDef(Rc::new(class_def))
}

fn new_native(name: &str, arity: Option<usize>, func: NativeFn) -> Value {
    Value::NativeFunction(Rc::new(NativeFunction::new(name, arity, func)))
}
