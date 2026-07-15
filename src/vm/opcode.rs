pub enum OpCode {
    Constant,
    Pop,

    Add,
    Sub,
    Mult,
    Div,

    StoreGlobal,
    StoreLocal,
    LoadGlobal,
    LoadLocal,
}
