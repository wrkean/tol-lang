use std::mem;

use crate::{
    analyze::symbol::{Storage, SymbolId},
    global_ctx::GlobalContext,
    module::{Module, ModuleId},
    parse::ast::{
        expr::{Expr, ExprKind},
        stmt::{Stmt, StmtKind},
    },
    vm::{chunk::Chunk, opcode::OpCode, value::Value},
};

/// Compiles the target module into chunks of bytecode
pub struct BytecodeCompiler<'gctx> {
    ctx: &'gctx GlobalContext,
    module_id: ModuleId,
    chunk: Chunk,
}

impl<'gctx> BytecodeCompiler<'gctx> {
    /// Create a new BytecodeCompiler with a target module
    pub fn new(ctx: &'gctx GlobalContext, module_id: ModuleId) -> Self {
        Self {
            ctx,
            module_id,
            chunk: Chunk::new(),
        }
    }

    /// Runs the compiler for the target module
    pub fn compile(&mut self) -> Chunk {
        let ast = self.ctx.module_by_id(self.module_id).ast();
        for statement in ast {
            self.compile_statement(statement);
        }
        self.chunk.emit_opcode(OpCode::Halt, 0);

        mem::take(&mut self.chunk)
    }

    fn compile_statement(&mut self, statement: &Stmt) {
        match statement.kind() {
            StmtKind::Ang { .. } => self.compile_ang(statement),
            StmtKind::Expr { expr } => self.compile_expression_statement(statement),
        }
    }

    fn compile_ang(&mut self, ang: &Stmt) {
        let StmtKind::Ang {
            name,
            is_mutable,
            ty,
            rhs,
        } = ang.kind()
        else {
            unreachable!()
        };

        self.compile_expression(rhs);
        let id = ang.symbol_id();
        let line = self.current_module().line_of(ang.span().start);
        self.store_symbol(id, line);
    }

    fn store_symbol(&mut self, symbol_id: SymbolId, line: usize) {
        let symbol = self.ctx.symbol_by_id(symbol_id);
        match symbol.storage() {
            Storage::Global(slot) => {
                self.chunk.emit_opcode(OpCode::StoreGlobal, line);
                self.chunk.emit_byte(*slot as u8, line);
            }
            Storage::Local(slot) => {
                self.chunk.emit_opcode(OpCode::StoreLocal, line);
                self.chunk.emit_byte(*slot as u8, line);
            }
        }
    }

    fn compile_expression_statement(&mut self, expr_stmt: &Stmt) {
        let StmtKind::Expr { expr } = expr_stmt.kind() else {
            unreachable!()
        };

        let current_module = self.current_module();
        let line = current_module.line_of(expr_stmt.span().start);
        self.compile_expression(expr);

        // This is an expression statement, we discard the value of the expression afterwards
        self.chunk.emit_opcode(OpCode::Pop, line);
    }

    fn compile_expression(&mut self, expression: &Expr) {
        let line = self.current_module().line_of(expression.span().start);
        match expression.kind() {
            ExprKind::Integer(x) => self.chunk.add_and_emit_constant(Value::Int(*x), line),
            ExprKind::Float(x) => self.chunk.add_and_emit_constant(Value::Float(*x), line),
            ExprKind::Identifier(ident) => {
                let id = expression.symbol_id();
                let symbol = self.ctx.symbol_by_id(id);
                match symbol.storage() {
                    Storage::Global(slot) => {
                        self.chunk.emit_opcode(OpCode::LoadGlobal, line);
                        self.chunk.emit_byte(*slot as u8, line);
                    }
                    Storage::Local(slot) => {
                        self.chunk.emit_opcode(OpCode::LoadLocal, line);
                        self.chunk.emit_byte(*slot as u8, line);
                    }
                }
            }
            ExprKind::Binary { left, right, op } => {
                let line = self.current_module().line_of(op.span().start);

                self.compile_expression(left);
                self.compile_expression(right);
                self.chunk.emit_operator(op.kind(), line);
            }
        }
    }

    fn current_module(&self) -> &Module {
        self.ctx.module_by_id(self.module_id)
    }
}
