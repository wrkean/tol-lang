pub enum OpCode {
    Constant,
    Pop,
    Halt,

    Add,
    Sub,
    Mult,
    Div,

    StoreGlobal,
    StoreLocal,
    LoadGlobal,
    LoadLocal,
}
