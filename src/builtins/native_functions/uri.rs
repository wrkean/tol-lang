use std::{collections::HashMap, rc::Rc};

use crate::{
    builtins::{
        self,
        methods::{self},
        native_functions::{NativeFn, NativeFunction},
    },
    global_ctx::GlobalContext,
    module::ModuleId,
    tol::diagnostic::runtime::RuntimeError,
    vm::{VM, class::ClassDef, value::Value},
};

macro_rules! native_class {
    ($name:literal, $( $method:literal => $arity:expr => $func:path ),* $(,)?) => {{
        let mut methods = HashMap::new();
        $( methods.insert($method.into(), new_native($method, $arity, $func)); )*
        Value::ClassDef(Rc::new(ClassDef::new($name.to_string(), methods)))
    }};
}

pub fn initialize_uri_module(target_module_id: ModuleId, ctx: &mut GlobalContext) {
    let module = ctx.module_by_id_mut(target_module_id);

    let lista_type = initialize_lista_type();
    let numero_type = initialize_numero_type();
    let teksto_type = initialize_teksto_type();
    let sakop_type = initialize_sakop_type();
    let iterator_type = initialize_iterator_type();

    module.new_global("Lista", lista_type);
    module.new_global("Numero", numero_type);
    module.new_global("Teksto", teksto_type);
    module.new_global("Sakop", sakop_type);
    module.new_global("Iterator", iterator_type);
    module.add_native_fn("anongUri", Some(1), native_anong_uri);
}

fn initialize_lista_type() -> Value {
    use builtins::methods::lista::*;
    native_class!("Lista",
        "dagdag" => Some(2) => dagdag,
        "haba" => Some(1) => haba,
        "bago" => Some(0) => bago,
        "iter" => Some(1) => iter,
    )
}

fn initialize_numero_type() -> Value {
    use builtins::methods::numero::*;
    native_class!("Numero",
        "abs" => Some(1) => abs,
        "bilangLutang" => Some(1) => bilang_lutang,
        "bilangString" => Some(1) => bilang_string,
        "bilangKarakter" => Some(1) => bilang_karakter,
    )
}

fn initialize_teksto_type() -> Value {
    use builtins::methods::string::*;
    native_class!("Teksto",
        "haba" => Some(1) => haba,
        "bilangNumero" => Some(1) => bilang_numero,
        "titik" => Some(1) => titik,
    )
}

fn initialize_sakop_type() -> Value {
    use builtins::methods::range::*;
    native_class!("Sakop",
        "hakbang" => Some(2) => hakbang,
        "iter" => Some(1) => iter,
    )
}

fn initialize_iterator_type() -> Value {
    use builtins::methods::iterator::*;
    native_class!("Iterator",
        "iMap" => Some(2) => map,
        "ipuninSaLista" => Some(1) => ipunin_sa_lista,
    )
}

fn insert(methods: &mut HashMap<String, Value>, name: &str, arity: Option<usize>, func: NativeFn) {
    methods.insert(name.into(), new_native(name, arity, func));
}

fn new_native(name: &str, arity: Option<usize>, func: NativeFn) -> Value {
    Value::NativeFunction(Rc::new(NativeFunction::new(name, arity, func)))
}

pub fn native_anong_uri(vm: &mut VM, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
    builtins::expected_args_count(vm, args.len(), 1)?;

    let type_name = match args.first().unwrap() {
        Value::Int(_) => "Numero",
        Value::Float(_) => "Lutang",
        Value::Bool(_) => "Bool",
        Value::Str(_) => "Teksto",
        Value::List(_) => "Lista",
        Value::Range(_) => "Sakop",
        Value::Iterator(_) => "Iterator",
        Value::ClassDef(def) => &def.name,
        Value::ClassInstance(inst) => &inst.borrow().def.name,
        Value::ModuleObj(module_obj) => "Module",
        Value::Function(_)
        | Value::Closure(_)
        | Value::NativeFunction(_)
        | Value::BoundMethod(_) => "Paraan",
        Value::Null => "Wala",
    };

    Ok(Value::Str(vm.intern_string(type_name)))
}
