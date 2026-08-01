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

// TODO: Very tedious!!! need a derive macro

pub fn initialize_uri_module(target_module_id: ModuleId, ctx: &mut GlobalContext) {
    let module = ctx.module_by_id_mut(target_module_id);

    let lista_type = initialize_lista_type();
    let numero_type = initialize_numero_type();
    let teksto_type = initialize_teksto_type();

    module.new_global("Lista", lista_type);
    module.new_global("Numero", numero_type);
    module.new_global("Teksto", teksto_type);
}

fn initialize_lista_type() -> Value {
    let mut methods = HashMap::new();

    use builtins::methods::lista::*;
    insert(&mut methods, "dagdag", Some(2), dagdag);
    insert(&mut methods, "haba", Some(1), haba);
    insert(&mut methods, "bago", Some(0), bago);
    insert(&mut methods, "__maging_iter__", Some(1), __maging_iter__);

    let class_def = ClassDef::new("Lista".to_string(), methods);
    Value::ClassDef(Rc::new(class_def))
}

fn initialize_numero_type() -> Value {
    let mut methods = HashMap::new();

    use builtins::methods::numero::*;
    insert(&mut methods, "abs", Some(1), abs);
    insert(&mut methods, "bilang_string", Some(1), bilang_string);
    insert(&mut methods, "bilang_karakter", Some(1), bilang_karakter);

    let class_def = ClassDef::new("Numero".to_string(), methods);
    Value::ClassDef(Rc::new(class_def))
}

fn initialize_teksto_type() -> Value {
    let mut methods = HashMap::new();

    use builtins::methods::string::*;
    insert(&mut methods, "haba", Some(1), haba);
    insert(&mut methods, "bilang_numero", Some(1), bilang_numero);
    insert(&mut methods, "titik", Some(1), titik);

    let class_def = ClassDef::new("Teksto".to_string(), methods);
    Value::ClassDef(Rc::new(class_def))
}

fn insert(methods: &mut HashMap<String, Value>, name: &str, arity: Option<usize>, func: NativeFn) {
    methods.insert(name.into(), new_native(name, arity, func));
}

fn new_native(name: &str, arity: Option<usize>, func: NativeFn) -> Value {
    Value::NativeFunction(Rc::new(NativeFunction::new(name, arity, func)))
}
