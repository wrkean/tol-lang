use crate::{
    tol::token::{Span, TokenKind},
    vm::{opcode::OpCode, value::Value},
};

#[derive(Debug, Clone)]
struct SpanRun {
    span: Span,
    count: usize,
}

#[derive(Default, Debug, Clone)]
pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    lines: Vec<SpanRun>,
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
    pub fn emit_opcode(&mut self, opcode: OpCode, span: Span) {
        self.write(opcode as u8, span);
    }

    /// Emits and stores a raw byte into the bytecode list
    pub fn emit_byte(&mut self, byte: u8, span: Span) {
        self.write(byte, span);
    }

    /// Emits and stores an unsigned 16-bit integer into the bytecode list
    pub fn emit_u16(&mut self, u16_: u16, span: Span) {
        self.write((u16_ >> 8) as u8, span.clone());
        self.write((u16_ & 0xFF) as u8, span);
    }

    pub fn add_constant(&mut self, constant: Value) -> u8 {
        self.constants.push(constant);

        (self.constants.len() - 1) as u8
    }

    pub fn get_constant(&self, constant_index: usize) -> Value {
        self.constants[constant_index].clone()
    }

    pub fn emit_constant(&mut self, index: u8, span: Span) {
        self.emit_opcode(OpCode::Constant, span.clone());
        self.emit_byte(index, span);
    }

    pub fn add_and_emit_constant(&mut self, value: Value, span: Span) {
        let index = self.add_constant(value);
        self.emit_opcode(OpCode::Constant, span.clone());
        self.emit_byte(index, span);
    }

    pub fn emit_operator(&mut self, op_kind: &TokenKind, span: Span) {
        let opcode = match op_kind {
            TokenKind::Plus => OpCode::Add,
            TokenKind::PlusEq => OpCode::AddEq,
            TokenKind::PlusPlus => OpCode::Concat,
            TokenKind::Minus => OpCode::Sub,
            TokenKind::MinusEq => OpCode::SubEq,
            TokenKind::Star => OpCode::Mult,
            TokenKind::StarEq => OpCode::MultEq,
            TokenKind::Slash => OpCode::Div,
            TokenKind::SlashEq => OpCode::DivEq,
            TokenKind::Percent => OpCode::Modulo,
            TokenKind::PercentEq => OpCode::ModuloEq,
            TokenKind::EqualEq => OpCode::EqualEq,
            TokenKind::NotEq => OpCode::NotEq,
            TokenKind::Greater => OpCode::Greater,
            TokenKind::GreatEq => OpCode::GreatEq,
            TokenKind::Lesser => OpCode::Lesser,
            TokenKind::LessEq => OpCode::LessEq,
            _ => unimplemented!(),
        };
        self.emit_opcode(opcode, span);
    }

    pub fn emit_jump(&mut self, jump_op: OpCode, span: Span) -> usize {
        self.emit_opcode(jump_op, span.clone());

        // Fill with placeholders (0xDEAD)
        self.emit_byte(0xDE, span.clone());
        self.emit_byte(0xAD, span);

        self.code.len() - 2
    }

    pub fn emit_loop(&mut self, loop_start: usize, span: Span) {
        self.emit_opcode(OpCode::Loop, span.clone());

        let offset = self.code.len() - loop_start + 2;

        if offset > u16::MAX as usize {
            panic!("Loop body too large.");
        }

        self.emit_byte((offset >> 8) as u8, span.clone());
        self.emit_byte(offset as u8, span);
    }

    pub fn patch_jump(&mut self, offset: usize) {
        let jump = self.code.len() - offset - 2;

        if jump > u16::MAX as usize {
            panic!("Too much code to jump over")
        }

        self.code[offset] = ((jump >> 8) & 0xFF) as u8;
        self.code[offset + 1] = (jump & 0xFF) as u8;
    }

    /// Disassemble this chunk, prints each bytecode with its corresponding information
    pub fn disassemble(&self, name: &str) {
        // println!("=== {name} ===");
        //
        // let mut offset = 0;
        // while offset < self.code.len() {
        //     offset = self.disassemble_instruction(offset);
        // }
    }

    pub fn get_byte(&self, index: usize) -> u8 {
        self.code[index]
    }

    pub fn ends_with_return(&self) -> bool {
        self.code
            .last()
            .is_some_and(|last| *last == OpCode::Return as u8)
    }

    // Helper function responsible for writing into the bytecode list `self.code`
    // It writes a byte (can be an opcode or a raw byte) and records it's line
    fn write(&mut self, byte: u8, span: Span) -> usize {
        self.code.push(byte);

        match self.lines.last_mut() {
            // Similar to self.lines.last_mut().is_some_and(|run| run.line == line)
            Some(run) if run.span == span => {
                run.count += 1;
            }
            _ => self.lines.push(SpanRun { span, count: 1 }),
        }

        self.code.len() - 1 // Return the index to where the byte was written
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04}    ", offset);

        let span = self.span_of(offset);

        if offset > 0 && span == self.span_of(offset - 1) {
            print!("    | ");
        } else {
            print!("{:?} ", span);
        }

        let instruction = self.code[offset];
        match instruction {
            op if op == OpCode::Add as u8 => self.simple_instruction("ADD", offset),
            op if op == OpCode::Concat as u8 => self.simple_instruction("CONCAT", offset),
            op if op == OpCode::Sub as u8 => self.simple_instruction("SUB", offset),
            op if op == OpCode::Mult as u8 => self.simple_instruction("MULT", offset),
            op if op == OpCode::Div as u8 => self.simple_instruction("DIV", offset),
            op if op == OpCode::EqualEq as u8 => self.simple_instruction("EQUAL_EQ", offset),
            op if op == OpCode::NotEq as u8 => self.simple_instruction("NOT_EQ", offset),
            op if op == OpCode::Greater as u8 => self.simple_instruction("GREATER", offset),
            op if op == OpCode::GreatEq as u8 => self.simple_instruction("GREAT_EQ", offset),
            op if op == OpCode::Lesser as u8 => self.simple_instruction("LESSER", offset),
            op if op == OpCode::LessEq as u8 => self.simple_instruction("LESS_EQ", offset),
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
            op if op == OpCode::Call as u8 => self.disassemble_byte_instruction("CALL", offset),
            op if op == OpCode::JumpIfFalse as u8 => {
                self.disassemble_2byte_instruction("JUMP_IF_FALSE", offset)
            }
            op if op == OpCode::Jump as u8 => self.disassemble_2byte_instruction("JUMP", offset),
            op if op == OpCode::LoadUpvalue as u8 => {
                self.disassemble_byte_instruction("LOAD_UPVALUE", offset)
            }
            op if op == OpCode::StoreUpvalue as u8 => {
                self.disassemble_byte_instruction("STORE_UPVALUE", offset)
            }
            op if op == OpCode::Null as u8 => self.simple_instruction("NULL", offset),
            op if op == OpCode::Return as u8 => self.simple_instruction("RETURN", offset),
            op if op == OpCode::Loop as u8 => self.disassemble_2byte_instruction("LOOP", offset),
            op if op == OpCode::NewClassInst as u8 => {
                self.disassemble_byte_instruction("NEW_CLASS_INST", offset)
            }
            op if op == OpCode::GetField as u8 => self.simple_instruction("GET_FIELD", offset),
            op if op == OpCode::SetField as u8 => self.simple_instruction("SET_FIELD", offset),
            op if op == OpCode::Closure as u8 => {
                let mut offset = offset + 1;
                let constant = self.code[offset];
                offset += 1;
                print!("{:-16} {:4}", "CLOSURE", constant);
                println!("{}", self.constants[constant as usize]);

                let upvalue_count = self.code[offset];
                offset += 1;
                for _ in 0..upvalue_count {
                    let is_local = self.code[offset] == 1;
                    let index = self.code[offset + 1];
                    println!(
                        "{:04}    |                     {} {}",
                        offset,
                        if is_local { "local" } else { "upvalue" },
                        index
                    );
                    offset += 2;
                }

                offset
            }
            _ => {
                println!("UNKNOWN OPCODE {:02X} AT OFFSET {:04}", instruction, offset);
                offset + 1
            }
        }
    }

    fn disassemble_byte_instruction(&self, name: &str, offset: usize) -> usize {
        let operand = self.code[offset + 1];
        println!("{:<16} {:4}", name, operand);
        offset + 2
    }

    fn disassemble_2byte_instruction(&self, name: &str, offset: usize) -> usize {
        let bytes = &self.code[offset + 1..offset + 3];
        let operand = u16::from_be_bytes([bytes[0], bytes[1]]);
        println!("{:<16} {:4}", name, operand);

        offset + 3
    }

    fn disassemble_constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant_index = self.code[offset + 1] as usize;
        let constant = self.constants.get(constant_index);

        match constant {
            Some(value) => println!("{:<16} {:4} Value({})", name, constant_index, value),
            None => println!("{:<16} {:4} <invalid constant>", name, constant_index),
        }

        offset + 2
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }

    pub fn span_of(&self, instruction: usize) -> Span {
        let mut current = 0;

        for run in self.lines.iter() {
            current += run.count;

            if instruction < current {
                return run.span.clone();
            }
        }

        panic!("Invalid instruction offset")
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }
}
