use crate::{
    tol::token::TokenKind,
    vm::{opcode::OpCode, value::Value},
};

#[derive(Debug, Clone)]
struct LineRun {
    line: usize,
    count: usize,
}

#[derive(Default)]
pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    lines: Vec<LineRun>,
}
impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    /// Emits and stores an opcode into the bytecode list
    pub fn emit_opcode(&mut self, opcode: OpCode, line: usize) {
        self.write(opcode as u8, line);
    }

    /// Emits and stores a raw byte into the bytecode list
    pub fn emit_byte(&mut self, byte: u8, line: usize) {
        self.write(byte, line);
    }

    pub fn add_constant(&mut self, constant: Value) -> u8 {
        self.constants.push(constant);

        (self.constants.len() - 1) as u8
    }

    pub fn get_constant(&self, constant_index: usize) -> Value {
        self.constants[constant_index].clone()
    }

    pub fn emit_constant(&mut self, index: u8, line: usize) {
        self.emit_opcode(OpCode::Constant, line);
        self.emit_byte(index, line);
    }

    pub fn add_and_emit_constant(&mut self, value: Value, line: usize) {
        let index = self.add_constant(value);
        self.emit_opcode(OpCode::Constant, line);
        self.emit_byte(index, line);
    }

    pub fn emit_operator(&mut self, op_kind: &TokenKind, line: usize) {
        let opcode = match op_kind {
            TokenKind::Plus => OpCode::Add,
            TokenKind::Minus => OpCode::Sub,
            TokenKind::Star => OpCode::Mult,
            TokenKind::Slash => OpCode::Div,
            _ => unimplemented!(),
        };
        self.emit_opcode(opcode, line);
    }

    /// Disassemble this chunk, prints each bytecode with its corresponding information
    pub fn disassemble(&self, name: &str) {
        println!("=== {name} ===");

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn get_byte(&self, index: usize) -> u8 {
        self.code[index]
    }

    // Helper function responsible for writing into the bytecode list `self.code`
    // It writes a byte (can be an opcode or a raw byte) and records it's line
    fn write(&mut self, byte: u8, line: usize) -> usize {
        self.code.push(byte);

        match self.lines.last_mut() {
            // Similar to self.lines.last_mut().is_some_and(|run| run.line == line)
            Some(run) if run.line == line => {
                run.count += 1;
            }
            _ => self.lines.push(LineRun { line, count: 1 }),
        }

        self.code.len() - 1 // Return the index to where the byte was written
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04}    ", offset);

        let line = self.line(offset);

        if offset > 0 && line == self.line(offset - 1) {
            print!("       | ");
        } else {
            print!("{:8} ", line);
        }

        let instruction = self.code[offset];
        match instruction {
            op if op == OpCode::Add as u8 => self.simple_instruction("ADD", offset),
            op if op == OpCode::Sub as u8 => self.simple_instruction("SUB", offset),
            op if op == OpCode::Mult as u8 => self.simple_instruction("MULT", offset),
            op if op == OpCode::Div as u8 => self.simple_instruction("DIV", offset),
            op if op == OpCode::Pop as u8 => self.simple_instruction("POP", offset),
            op if op == OpCode::Halt as u8 => self.simple_instruction("HALT", offset),
            op if op == OpCode::Print as u8 => self.simple_instruction("PRINT", offset),
            op if op == OpCode::Constant as u8 => {
                self.disassemble_constant_instruction("CONSTANT", offset)
            }
            op if op == OpCode::StoreGlobal as u8 => {
                self.disassemble_byte_instruction("STORE_GLOBAL", offset)
            }
            op if op == OpCode::StoreLocal as u8 => {
                self.disassemble_byte_instruction("STORE_LOCAL", offset)
            }
            op if op == OpCode::LoadGlobal as u8 => {
                self.disassemble_byte_instruction("LOAD_GLOBAL", offset)
            }
            op if op == OpCode::LoadLocal as u8 => {
                self.disassemble_byte_instruction("LOAD_LOCAL", offset)
            }
            _ => {
                println!("UNKNOWN OPCODE: {:02X}", offset);
                offset + 1
            }
        }
    }

    fn disassemble_byte_instruction(&self, name: &str, offset: usize) -> usize {
        let operand = self.code[offset + 1];
        println!("{:<16} {:4}", name, operand);
        offset + 2
    }

    fn disassemble_constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant_index = self.code[offset + 1] as usize;
        let constant = self.constants.get(constant_index);

        match constant {
            Some(value) => println!("{:<16} {:4} {:?}", name, constant_index, value),
            None => println!("{:<16} {:4} <invalid constant>", name, constant_index),
        }

        offset + 2
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }

    fn line(&self, instruction: usize) -> usize {
        let mut current = 0;

        for run in self.lines.iter() {
            current += run.count;

            if instruction < current {
                return run.line;
            }
        }

        panic!("Invalid instruction offset")
    }
}
