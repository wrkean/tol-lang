use std::{mem, rc::Rc};

use crate::{
    analyze::symbol::{Storage, SymbolId},
    global_ctx::GlobalContext,
    module::{Module, ModuleId},
    parse::ast::{
        expr::{Expr, ExprKind},
        stmt::{Stmt, StmtKind},
    },
    tol::token::TokenKind,
    vm::{chunk::Chunk, function::Function, opcode::OpCode, value::Value},
};

struct LoopContext {
    break_jumps: Vec<usize>,
    loop_start: usize,
}

/// Compiles the target module into chunks of bytecode
pub struct BytecodeCompiler<'gctx> {
    ctx: &'gctx GlobalContext,
    module_id: ModuleId,
    chunk: Chunk,
    loop_stack: Vec<LoopContext>,
}

impl<'gctx> BytecodeCompiler<'gctx> {
    /// Create a new BytecodeCompiler with a target module
    pub fn new(ctx: &'gctx GlobalContext, module_id: ModuleId) -> Self {
        Self {
            ctx,
            module_id,
            chunk: Chunk::new(),
            loop_stack: Vec::new(),
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
            StmtKind::Paraan { .. } => self.compile_paraan(statement),
            StmtKind::Print { .. } => self.compile_print(statement),
            StmtKind::Kung { .. } => self.compile_kung(statement),
            StmtKind::Habang { .. } => self.compile_habang(statement),
            StmtKind::Biyakin => self.compile_biyakin(statement),
            StmtKind::Ituloy => self.compile_ituloy(statement),
            StmtKind::Ibalik { .. } => self.compile_ibalik(statement),
            StmtKind::Expr { .. } => self.compile_expression_statement(statement),
            StmtKind::Block { statements } => {
                for statement in statements {
                    self.compile_statement(statement);
                }
            }
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

    fn compile_paraan(&mut self, paraan: &Stmt) {
        let StmtKind::Paraan {
            name,
            params,
            ret_ty,
            block,
        } = paraan.kind()
        else {
            unreachable!()
        };

        let symbol = self.ctx.symbol_by_id(paraan.symbol_id());

        // Temporarily replace current chunk with a new chunk assumed to be chunks produced by the blocks of this function
        let mut function_chunk = Chunk::new();
        let old_chunk = mem::replace(&mut self.chunk, function_chunk);

        self.compile_statement(block);
        let line = self.current_module().line_of(block.span().end);
        if !self.chunk.ends_with_return() {
            self.chunk.emit_opcode(OpCode::Null, line);
            self.chunk.emit_opcode(OpCode::Return, line);
        }

        // After compiling the block, we put the chunk back to its place and retrieve the chunks
        // produced by compiling the function block
        function_chunk = mem::replace(&mut self.chunk, old_chunk);

        let TokenKind::Identifier(function_name) = name.kind() else {
            unreachable!()
        };
        let function = Function::new(function_name.clone(), function_chunk, params.len() as u8);

        let line = self.current_module().line_of(paraan.span().start);
        self.chunk
            .add_and_emit_constant(Value::Function(Rc::new(function)), line);
        self.store_symbol(paraan.symbol_id(), line);
    }

    fn compile_print(&mut self, print: &Stmt) {
        let StmtKind::Print { expr } = print.kind() else {
            unreachable!()
        };

        let line = self.current_module().line_of(print.span().start);
        self.compile_expression(expr);
        self.chunk.emit_opcode(OpCode::Print, line);
    }

    fn compile_kung(&mut self, kung: &Stmt) {
        let StmtKind::Kung {
            then_branches,
            else_branch,
        } = kung.kind()
        else {
            unreachable!()
        };

        let mut end_jumps = Vec::new();
        for then in then_branches {
            let condition = then.condition.as_ref().unwrap();
            let block = &then.block;
            let cond_line = self.current_module().line_of(condition.span().start);
            self.compile_expression(condition);
            let jump_if_false = self.chunk.emit_jump(OpCode::JumpIfFalse, cond_line);
            self.chunk.emit_opcode(OpCode::Pop, cond_line);

            let block_line = self.current_module().line_of(block.span().start);
            self.compile_statement(block);
            end_jumps.push(self.chunk.emit_jump(OpCode::Jump, block_line));
            self.chunk.patch_jump(jump_if_false);
            self.chunk.emit_opcode(OpCode::Pop, cond_line);
        }

        if let Some(branch) = else_branch {
            self.compile_statement(&branch.block);
        }

        for end in end_jumps {
            self.chunk.patch_jump(end);
        }
    }

    fn compile_habang(&mut self, habang: &Stmt) {
        let StmtKind::Habang { condition, block } = habang.kind() else {
            unreachable!()
        };

        let loop_start = self.chunk.code().len();

        self.loop_stack.push(LoopContext {
            loop_start,
            break_jumps: Vec::new(),
        });

        let line = self.current_module().line_of(condition.span().start);

        self.compile_expression(condition);
        let exit_jump = self.chunk.emit_jump(OpCode::JumpIfFalse, line);
        self.chunk.emit_opcode(OpCode::Pop, line);

        self.compile_statement(block);
        self.chunk.emit_loop(loop_start, line);
        self.chunk.patch_jump(exit_jump);
        self.chunk.emit_opcode(OpCode::Pop, line);

        let ctx = self.loop_stack.pop().unwrap();
        for jump in ctx.break_jumps {
            self.chunk.patch_jump(jump);
        }
    }

    fn compile_biyakin(&mut self, biyakin: &Stmt) {
        let line = self.current_module().line_of(biyakin.span().start);
        let jump = self.chunk.emit_jump(OpCode::Jump, line);
        let loop_ctx = self.loop_stack.last_mut().unwrap();
        loop_ctx.break_jumps.push(jump);
    }

    fn compile_ituloy(&mut self, ituloy: &Stmt) {
        let line = self.current_module().line_of(ituloy.span().start);
        let loop_ctx = self.loop_stack.last().unwrap();
        self.chunk.emit_loop(loop_ctx.loop_start, line);
    }

    fn compile_ibalik(&mut self, ibalik: &Stmt) {
        let StmtKind::Ibalik { expr } = ibalik.kind() else {
            unreachable!()
        };

        let line = self.current_module().line_of(ibalik.span().start);
        match expr {
            Some(ex) => self.compile_expression(ex),
            None => self.chunk.add_and_emit_constant(Value::Null, line),
        }

        self.chunk.emit_opcode(OpCode::Return, line);
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
            ExprKind::Str(s) => self
                .chunk
                .add_and_emit_constant(Value::Str(Rc::from(s.as_str())), line),
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

                if op.kind() == &TokenKind::Equal {
                    self.compile_assignment(expression);
                } else {
                    self.compile_expression(left);
                    self.compile_expression(right);
                    self.chunk.emit_operator(op.kind(), line);
                }
            }
            ExprKind::Call { left, args } => {
                self.compile_expression(left);

                for arg in args {
                    self.compile_expression(arg);
                }

                let line = self.current_module().line_of(left.span().start);
                self.chunk.emit_opcode(OpCode::Call, line);
                self.chunk.emit_byte(args.len() as u8, line);
            }
        }
    }

    fn compile_assignment(&mut self, assignment: &Expr) {
        let ExprKind::Binary { left, right, op } = assignment.kind() else {
            unreachable!()
        };

        self.compile_expression(right);
        let line = self.current_module().line_of(assignment.span().start);
        self.store_symbol(left.symbol_id(), line);

        self.chunk.emit_opcode(OpCode::Null, line);
    }

    fn current_module(&self) -> &Module {
        self.ctx.module_by_id(self.module_id)
    }
}
