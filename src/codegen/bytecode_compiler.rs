use std::{cmp::Reverse, mem, rc::Rc};

use crate::{
    analyze::{
        ResolvedVar,
        symbol::{Storage, SymbolId, SymbolKind},
    },
    global_ctx::GlobalContext,
    module::{Module, ModuleId},
    parse::ast::{
        expr::{Expr, ExprKind},
        stmt::{Stmt, StmtKind},
    },
    tol::token::{Span, TokenKind},
    vm::{chunk::Chunk, class::ClassDef, function::Function, opcode::OpCode, value::Value},
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
        self.load_dependencies();
        let ast = self.ctx.module_by_id(self.module_id).ast();
        for statement in ast {
            self.compile_statement(statement);
        }

        let span = {
            if let Some(last) = ast.last() {
                last.span().clone()
            } else {
                0..0
            }
        };
        self.chunk.emit_opcode(OpCode::Null, span.clone());
        self.chunk.emit_opcode(OpCode::Return, span);

        mem::take(&mut self.chunk)
    }

    pub fn load_dependencies(&mut self) {
        let current_module = self.current_module();
        for module_id in current_module.dependencies().to_vec() {
            let constant_index = self.chunk.add_constant(Value::Int(module_id as i64));
            self.chunk.emit_opcode(OpCode::ImportModule, 0..0);
            self.chunk.emit_byte(constant_index, 0..0);
            let current_module = self.current_module();
            let resolved_dependency = current_module.get_resolved_dependency(module_id);
            self.store_var(resolved_dependency.clone(), 0..0);
        }
    }

    fn compile_statement(&mut self, statement: &Stmt) {
        match statement.kind() {
            StmtKind::Ang { .. } => self.compile_ang(statement),
            StmtKind::Paraan { .. } => self.compile_paraan(statement),
            StmtKind::Kung { .. } => self.compile_kung(statement),
            StmtKind::Habang { .. } => self.compile_habang(statement),
            StmtKind::Biyakin => self.compile_biyakin(statement),
            StmtKind::Ituloy => self.compile_ituloy(statement),
            StmtKind::Ibalik { .. } => self.compile_ibalik(statement),
            StmtKind::Klase { .. } => self.compile_klase(statement),
            StmtKind::Kunin { .. } => self.compile_kunin(statement),
            StmtKind::Expr { .. } => self.compile_expression_statement(statement),
            StmtKind::Block { statements } => {
                for statement in statements {
                    self.compile_statement(statement);
                }
            }
        }
    }

    fn compile_ang(&mut self, ang: &Stmt) {
        let StmtKind::Ang { name, ty, rhs } = ang.kind() else {
            unreachable!()
        };

        self.compile_expression(rhs);
        let id = ang.resolved_var();
        let span = name.span().clone();
        self.store_var(id, span);
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

        // Temporarily replace current chunk with a new chunk assumed to be chunks produced by the blocks of this function
        let mut function_chunk = Chunk::new();
        let old_chunk = mem::replace(&mut self.chunk, function_chunk);

        self.compile_statement(block);
        let span = block.span().clone();
        if !self.chunk.ends_with_return() {
            self.chunk.emit_opcode(OpCode::Null, span.clone());
            self.chunk.emit_opcode(OpCode::Return, span);
        }

        // After compiling the block, we put the chunk back to its place and retrieve the chunks
        // produced by compiling the function block
        function_chunk = mem::replace(&mut self.chunk, old_chunk);

        let symbol = self.ctx.symbol_by_id(paraan.resolved_var().symbol_id());

        let function_name = name.lexeme();
        let function = Function::new(
            function_name.to_string(),
            function_chunk,
            params.len() as u8,
            symbol.frame_size(),
            self.module_id,
        );

        let SymbolKind::Function { upvalues, .. } = symbol.kind() else {
            unreachable!()
        };
        let span = paraan.span().clone();
        let const_index = self.chunk.add_constant(Value::Function(Rc::new(function)));
        self.chunk.emit_opcode(OpCode::Closure, span.clone());
        self.chunk.emit_byte(const_index, span.clone());
        self.chunk.emit_byte(upvalues.len() as u8, span.clone());
        for upvalue in upvalues {
            self.chunk
                .emit_byte(if upvalue.is_local { 1 } else { 0 }, span.clone());
            self.chunk.emit_byte(upvalue.index as u8, span.clone());
        }
        self.store_var(paraan.resolved_var(), span);
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
            let cond_span = condition.span().clone();
            self.compile_expression(condition);
            let jump_if_false = self.chunk.emit_jump(OpCode::JumpIfFalse, cond_span.clone());
            self.chunk.emit_opcode(OpCode::Pop, cond_span.clone());

            let block_span = block.span().clone();
            self.compile_statement(block);
            end_jumps.push(self.chunk.emit_jump(OpCode::Jump, block_span));
            self.chunk.patch_jump(jump_if_false);
            self.chunk.emit_opcode(OpCode::Pop, cond_span);
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

        let span = condition.span().clone();

        self.compile_expression(condition);
        let exit_jump = self.chunk.emit_jump(OpCode::JumpIfFalse, span.clone());
        self.chunk.emit_opcode(OpCode::Pop, span.clone());

        self.compile_statement(block);
        self.chunk.emit_loop(loop_start, span.clone());
        self.chunk.patch_jump(exit_jump);
        self.chunk.emit_opcode(OpCode::Pop, span);

        let ctx = self.loop_stack.pop().unwrap();
        for jump in ctx.break_jumps {
            self.chunk.patch_jump(jump);
        }
    }

    fn compile_biyakin(&mut self, biyakin: &Stmt) {
        let span = biyakin.span().clone();
        let jump = self.chunk.emit_jump(OpCode::Jump, span);
        let loop_ctx = self.loop_stack.last_mut().unwrap();
        loop_ctx.break_jumps.push(jump);
    }

    fn compile_ituloy(&mut self, ituloy: &Stmt) {
        let span = ituloy.span().clone();
        let loop_ctx = self.loop_stack.last().unwrap();
        self.chunk.emit_loop(loop_ctx.loop_start, span);
    }

    fn compile_ibalik(&mut self, ibalik: &Stmt) {
        let StmtKind::Ibalik { expr } = ibalik.kind() else {
            unreachable!()
        };

        let span = ibalik.span().clone();
        match expr {
            Some(ex) => self.compile_expression(ex),
            None => self.chunk.add_and_emit_constant(Value::Null, span.clone()),
        }

        self.chunk.emit_opcode(OpCode::Return, span);
    }

    fn compile_klase(&mut self, klase: &Stmt) {
        let StmtKind::Klase { name, methods } = klase.kind() else {
            unreachable!()
        };

        for method in methods {
            self.compile_method(method);
        }

        let name_id = self.ctx.intern(name.lexeme());
        self.chunk
            .add_and_emit_constant(Value::Str(name_id), name.span().clone());
        self.chunk
            .emit_opcode(OpCode::DefineClass, name.span().clone());
        self.chunk
            .emit_byte(methods.len() as u8, klase.span().clone());

        self.store_var(klase.resolved_var(), klase.span().clone());
    }

    fn compile_kunin(&mut self, kunin: &Stmt) {
        let StmtKind::Kunin {
            segments,
            import_path_type,
        } = kunin.kind()
        else {
            unreachable!()
        };

        let symbol = self.ctx.symbol_by_id(kunin.resolved_var().symbol_id());
        let SymbolKind::Module { module_id } = symbol.kind() else {
            unreachable!()
        };

        let constant_index = self.chunk.add_constant(Value::Int(*module_id as i64));
        self.chunk
            .emit_opcode(OpCode::ImportModule, kunin.span().clone());
        self.chunk.emit_byte(constant_index, kunin.span().clone());
        self.store_var(kunin.resolved_var(), kunin.span().clone());
    }

    fn compile_method(&mut self, method: &Stmt) {
        let StmtKind::Paraan { name, .. } = method.kind() else {
            unreachable!()
        };
        let name_id = self.ctx.intern(name.lexeme());
        self.chunk
            .add_and_emit_constant(Value::Str(name_id), name.span().clone());
        self.compile_paraan(method);
        self.load_var(method.resolved_var(), name.span().clone()); // Pushes the method into the stack
    }

    fn store_var(&mut self, var: ResolvedVar, span: Span) {
        match var {
            ResolvedVar::Global(symbol_id) | ResolvedVar::Local(symbol_id) => {
                let symbol = self.ctx.symbol_by_id(symbol_id);
                match symbol.storage() {
                    Storage::Global(slot) => {
                        self.store_in_global_slot(*slot, span);
                    }
                    Storage::Local(slot) => {
                        self.store_in_local_slot(*slot, span);
                    }
                }
            }
            ResolvedVar::Upvalue(upvalue_idx) => {
                self.chunk.emit_opcode(OpCode::StoreUpvalue, span.clone());
                self.chunk.emit_byte(upvalue_idx as u8, span);
            }
        }
    }

    fn store_in_global_slot(&mut self, slot: usize, span: Span) {
        self.chunk.emit_opcode(OpCode::StoreGlobal, span.clone());
        self.chunk.emit_byte(slot as u8, span);
    }

    fn store_in_local_slot(&mut self, slot: usize, span: Span) {
        self.chunk.emit_opcode(OpCode::StoreLocal, span.clone());
        self.chunk.emit_byte(slot as u8, span);
    }

    fn load_var(&mut self, var: ResolvedVar, span: Span) {
        match var {
            ResolvedVar::Global(symbol_id) | ResolvedVar::Local(symbol_id) => {
                let symbol = self.ctx.symbol_by_id(symbol_id);
                match symbol.storage() {
                    Storage::Global(slot) => {
                        self.chunk.emit_opcode(OpCode::LoadGlobal, span.clone());
                        self.chunk.emit_byte(*slot as u8, span);
                    }
                    Storage::Local(slot) => {
                        self.chunk.emit_opcode(OpCode::LoadLocal, span.clone());
                        self.chunk.emit_byte(*slot as u8, span);
                    }
                }
            }
            ResolvedVar::Upvalue(upvalue_idx) => {
                self.chunk.emit_opcode(OpCode::LoadUpvalue, span.clone());
                self.chunk.emit_byte(upvalue_idx as u8, span);
            }
        }
    }

    fn compile_expression_statement(&mut self, expr_stmt: &Stmt) {
        let StmtKind::Expr { expr } = expr_stmt.kind() else {
            unreachable!()
        };

        let current_module = self.current_module();
        let span = expr.span().clone();
        self.compile_expression(expr);

        // This is an expression statement, we discard the value of the expression afterwards
        self.chunk.emit_opcode(OpCode::Pop, span);
    }

    fn compile_expression(&mut self, expression: &Expr) {
        let span = expression.span().clone();
        match expression.kind() {
            ExprKind::Integer(x) => self.chunk.add_and_emit_constant(Value::Int(*x), span),
            ExprKind::Float(x) => self.chunk.add_and_emit_constant(Value::Float(*x), span),
            ExprKind::Str { interned_id, .. } => {
                self.chunk
                    .add_and_emit_constant(Value::Str(interned_id.unwrap()), span);
            }
            ExprKind::Identifier(ident) => self.load_var(expression.resolved_var(), span),
            ExprKind::Binary { left, right, op } => {
                let line = self.current_module().line_of(op.span().start);

                if matches!(
                    op.kind(),
                    TokenKind::Equal
                        | TokenKind::PlusEq
                        | TokenKind::MinusEq
                        | TokenKind::StarEq
                        | TokenKind::SlashEq
                        | TokenKind::PercentEq
                ) {
                    self.compile_assignment(expression);
                } else {
                    self.compile_expression(left);
                    self.compile_expression(right);
                    self.chunk.emit_operator(op.kind(), span.clone());
                }
            }
            ExprKind::Call { left, args } => {
                if let ExprKind::FieldAccess { object, field } = left.kind() {
                    // Will be replaced later by the instance or the class def
                    self.chunk.emit_opcode(OpCode::Null, span.clone());

                    // Compile the receiver
                    self.compile_expression(object);

                    for arg in args {
                        self.compile_expression(arg);
                    }

                    let field_name_id = self.ctx.intern(field.lexeme());
                    let name_span = field.span().clone();
                    let const_index = self.chunk.add_constant(Value::Str(field_name_id));

                    self.chunk.emit_opcode(OpCode::Invoke, span.clone());
                    self.chunk.emit_byte(const_index, name_span);
                    self.chunk
                        .emit_byte(args.len() as u8 + 1 /* includes the receiver */, span)
                } else {
                    self.compile_expression(left);

                    for arg in args {
                        self.compile_expression(arg);
                    }

                    let line = self.current_module().line_of(left.span().start);
                    self.chunk.emit_opcode(OpCode::Call, span.clone());
                    self.chunk.emit_byte(args.len() as u8, span);
                }
            }
            ExprKind::AnonymousFn { params, body } => {
                let symbol = self.ctx.symbol_by_id(expression.resolved_var().symbol_id());

                let mut function_chunk = Chunk::new();
                let old_chunk = mem::replace(&mut self.chunk, function_chunk);

                self.compile_expression(body);
                let line = self.current_module().line_of(body.span().end);
                self.chunk.emit_opcode(OpCode::Return, span.clone());

                function_chunk = mem::replace(&mut self.chunk, old_chunk);

                let function = Function::new(
                    format!(
                        "__anonymous_fn_{}_{}__",
                        expression.span().start,
                        expression.span().end
                    ),
                    function_chunk,
                    params.len() as u8,
                    symbol.frame_size(),
                    self.module_id,
                );

                let SymbolKind::Function { upvalues, .. } = symbol.kind() else {
                    unreachable!()
                };
                let const_index = self.chunk.add_constant(Value::Function(Rc::new(function)));
                self.chunk.emit_opcode(OpCode::Closure, span.clone());
                self.chunk.emit_byte(const_index, span.clone());
                self.chunk.emit_byte(upvalues.len() as u8, span.clone());
                for upvalue in upvalues {
                    self.chunk
                        .emit_byte(if upvalue.is_local { 1 } else { 0 }, span.clone());
                    self.chunk.emit_byte(upvalue.index as u8, span.clone());
                }
            }
            ExprKind::FieldAccess { object, field } => {
                self.compile_expression(object);

                let field_name_id = self.ctx.intern(field.lexeme());
                self.chunk
                    .add_and_emit_constant(Value::Str(field_name_id), field.span().clone());
                self.chunk
                    .emit_opcode(OpCode::GetField, field.span().clone());
            }
            ExprKind::List { elements, init } => {
                for element in elements.iter().rev() {
                    self.compile_expression(element);
                }

                match init {
                    Some(list_init) => {
                        let TokenKind::IntLiteral(capacity) = list_init.init_capacity.kind() else {
                            unreachable!()
                        };

                        self.chunk
                            .add_and_emit_constant(Value::Int(*capacity), span.clone());
                        self.compile_expression(&list_init.expr);
                        self.chunk.emit_opcode(OpCode::ListWithCapacity, span);
                    }
                    None => {
                        self.chunk.emit_opcode(OpCode::List, span.clone());
                        self.chunk.emit_u16(elements.len() as u16, span);
                    }
                }
            }
            ExprKind::IndexAccess { left, index } => {
                self.compile_expression(left);
                self.compile_expression(index);

                self.chunk
                    .emit_opcode(OpCode::IndexGet, index.span().clone());
            }
        }
    }

    fn compile_assignment(&mut self, assignment: &Expr) {
        let ExprKind::Binary { left, right, op } = assignment.kind() else {
            unreachable!()
        };

        match left.kind() {
            ExprKind::FieldAccess { object, field } => {
                if op.kind() != &TokenKind::Equal {
                    self.compile_expression(left);
                    self.compile_expression(right);
                    self.chunk.emit_operator(op.kind(), op.span().clone());

                    self.compile_expression(object);
                } else {
                    self.compile_expression(right);
                    self.compile_expression(object);
                }

                let field_name_id = self.ctx.intern(field.lexeme());
                self.chunk
                    .add_and_emit_constant(Value::Str(field_name_id), field.span().clone());
                self.chunk
                    .emit_opcode(OpCode::SetField, field.span().clone());
            }
            ExprKind::IndexAccess { left, index } => {
                if op.kind() != &TokenKind::Equal {
                    self.compile_expression(left);
                    self.compile_expression(index);
                    self.chunk.emit_opcode(OpCode::IndexGet, op.span().clone());

                    self.compile_expression(right);
                    self.chunk.emit_operator(op.kind(), op.span().clone());
                } else {
                    self.compile_expression(right);
                }
                self.compile_expression(left); // Target
                self.compile_expression(index); // Index

                self.chunk
                    .emit_opcode(OpCode::IndexSet, index.span().clone());
            }
            _ => {
                if op.kind() != &TokenKind::Equal {
                    self.compile_expression(left);
                    self.compile_expression(right);
                    self.chunk.emit_operator(op.kind(), op.span().clone());
                } else {
                    self.compile_expression(right);
                }
                let span = left.span().clone();
                self.store_var(left.resolved_var(), span.clone());
            }
        }

        self.chunk
            .emit_opcode(OpCode::Null, assignment.span().clone());
    }

    fn current_module(&self) -> &Module {
        self.ctx.module_by_id(self.module_id)
    }
}
