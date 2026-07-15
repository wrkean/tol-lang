pub enum OpCode {
    Constant,
    Print,
    StoreGlobal,
    StoreLocal,
    LoadGlobal,
    LoadLocal,

    Pop,
    Halt,
    Add,
    Sub,
    Mult,
    Div,
    Null,
}
