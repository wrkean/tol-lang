use std::collections::HashMap;

use crate::{
    analyze::symbol::{Storage, Symbol, SymbolId, SymbolKind},
    global_ctx::GlobalContext,
    module::{Module, ModuleId},
    parse::ast::{
        Ast,
        expr::{Expr, ExprKind},
        stmt::{Stmt, StmtKind},
    },
    prelude::DiagResult,
    tol::{
        diagnostic::{Label, TolDiagnostic, predefined_diagnostics},
        token::{Token, TokenKind},
    },
};

pub mod symbol;

type Scope = HashMap<String, SymbolId>;

/// Analyzes the target module's ast
pub struct Analyzer<'gctx> {
    scopes: Vec<Scope>,
    ctx: &'gctx mut GlobalContext,
    module_id: ModuleId,
    loop_depth: u8,

    next_global_slot: usize,
    next_local_slot: usize,
    next_local_slot_stack: Vec<usize>,
}

impl<'gctx> Analyzer<'gctx> {
    /// Creates a new analyze instance that targets the given module by id
    pub fn new(ctx: &'gctx mut GlobalContext, module_id: ModuleId) -> Self {
        Self {
            scopes: vec![Scope::new()],
            ctx,
            module_id,
            loop_depth: 0,
            next_global_slot: 0,
            next_local_slot: 0,
            next_local_slot_stack: Vec::new(),
        }
    }

    /// Runs the analyzer on the target module
    pub fn analyze(&mut self) {
        self.resolve_names();
    }

    fn resolve_names(&mut self) {
        let mut ast = self.current_module_mut().take_ast();
        for statement in ast.iter_mut() {
            if let Err(diag) = self.resolve_statement(statement) {
                self.current_module_mut().add_diagnostic(*diag);
            }
        }
        self.current_module_mut().set_ast(ast);
    }

    fn resolve_statement(&mut self, statement: &mut Stmt) -> DiagResult<()> {
        match statement.kind_mut() {
            StmtKind::Ang { .. } => self.resolve_ang(statement),
            StmtKind::Paraan { .. } => self.resolve_paraan(statement),
            StmtKind::Print { expr } => self.resolve_expression(expr),
            StmtKind::Expr { expr } => self.resolve_expression(expr),
            StmtKind::Kung { .. } => self.resolve_kung(statement),
            StmtKind::Habang { .. } => self.resolve_habang(statement),
            StmtKind::Biyakin => self.resolve_biyakin(statement),
            StmtKind::Ituloy => self.resolve_ituloy(statement),
            StmtKind::Ibalik { .. } => self.resolve_ibalik(statement),
            StmtKind::Block { statements } => {
                for statement in statements {
                    if let Err(diag) = self.resolve_statement(statement) {
                        self.current_module_mut().add_diagnostic(*diag);
                    }
                }

                Ok(())
            }
        }
    }

    fn resolve_ang(&mut self, ang: &mut Stmt) -> DiagResult<()> {
        let StmtKind::Ang {
            name,
            is_mutable,
            ty,
            rhs,
        } = ang.kind_mut()
        else {
            unreachable!()
        };

        self.resolve_expression(rhs)?;
        let storage = self.assign_storage();

        // Should be an identifier if the parser is a good boy
        let TokenKind::Identifier(symbol_name) = name.kind() else {
            unreachable!()
        };
        let symbol = Symbol::new(
            symbol_name.clone(),
            name.span().clone(),
            storage,
            SymbolKind::Name {
                is_mutable: *is_mutable,
                ty: ty.clone(),
            },
        );

        let id = self.declare_symbol(symbol)?;

        // This ast node is now pointing it's symbol id to its declaration in the symbol table
        ang.set_symbol_id(id);

        Ok(())
    }

    fn resolve_paraan(&mut self, paraan: &mut Stmt) -> DiagResult<()> {
        let StmtKind::Paraan {
            name,
            params,
            ret_ty,
            block,
        } = paraan.kind_mut()
        else {
            unreachable!()
        };

        let storage = self.assign_storage();
        let TokenKind::Identifier(symbol_name) = name.kind() else {
            unreachable!()
        };
        let symbol = Symbol::new(
            symbol_name.clone(),
            name.span().clone(),
            storage,
            SymbolKind::Function {
                param_types: params.spanned_types(),
                ret_ty: ret_ty.clone(),
            },
        );
        let id = self.declare_symbol(symbol)?;

        self.enter_function();
        self.enter_scope();

        for param in params.params.iter() {
            let TokenKind::Identifier(param_name) = param.name.kind() else {
                unreachable!()
            };
            let symbol = Symbol::new(
                param_name.clone(),
                param.span.clone(),
                self.assign_storage(),
                SymbolKind::Name {
                    is_mutable: param.is_mutable,
                    ty: param.ty.clone(),
                },
            );

            if let Err(diag) = self.declare_symbol(symbol) {
                self.current_module_mut().add_diagnostic(*diag);
            }
        }

        self.enter_scope();
        self.resolve_statement(block)?;

        self.exit_function();
        self.exit_scope();
        self.exit_scope();

        paraan.set_symbol_id(id);

        Ok(())
    }

    fn resolve_kung(&mut self, kung: &mut Stmt) -> DiagResult<()> {
        let StmtKind::Kung {
            then_branches,
            else_branch,
        } = kung.kind_mut()
        else {
            unreachable!()
        };

        for then in then_branches {
            self.resolve_expression(then.condition.as_mut().unwrap())?;
            self.resolve_statement(&mut then.block)?;
        }

        if let Some(else_) = else_branch {
            self.resolve_statement(&mut else_.block)?;
        }

        Ok(())
    }

    fn resolve_habang(&mut self, habang: &mut Stmt) -> DiagResult<()> {
        let StmtKind::Habang { condition, block } = habang.kind_mut() else {
            unreachable!()
        };

        self.resolve_expression(condition)?;
        self.loop_depth += 1;
        self.resolve_statement(block)?;
        self.loop_depth -= 1;

        Ok(())
    }

    fn resolve_biyakin(&mut self, biyakin: &Stmt) -> DiagResult<()> {
        if self.loop_depth == 0 {
            let current_module = self.current_module();
            let diagnostic = TolDiagnostic::err(
                current_module.source_arc(),
                current_module.filename(),
                "paggamit ng `biyakin` sa labas ng loop",
            )
            .label(Label::new(biyakin.span().clone()).message("ito ay nasa labas ng loop"))
            .help("maaari lamang gamitin ang `biyakin` sa loob ng loop");

            return Err(Box::new(diagnostic));
        }

        Ok(())
    }

    fn resolve_ituloy(&mut self, ituloy: &Stmt) -> DiagResult<()> {
        if self.loop_depth == 0 {
            let current_module = self.current_module();
            let diagnostic = TolDiagnostic::err(
                current_module.source_arc(),
                current_module.filename(),
                "paggamit ng `ituloy` sa labas ng loop",
            )
            .label(Label::new(ituloy.span().clone()).message("ito ay nasa labas ng loop"))
            .help("maaari lamang gamitin ang `biyakin` sa loob ng loop");

            return Err(Box::new(diagnostic));
        }

        Ok(())
    }

    fn resolve_ibalik(&mut self, ibalik: &mut Stmt) -> DiagResult<()> {
        let StmtKind::Ibalik { expr } = ibalik.kind_mut() else {
            unreachable!()
        };

        if let Some(ex) = expr {
            self.resolve_expression(ex)?;
        }

        if self.next_local_slot_stack.is_empty() {
            let current_module = self.current_module();
            let diagnostic = TolDiagnostic::err(
                current_module.source_arc(),
                current_module.filename(),
                "paggamit ng `ibalik` sa labas ng paraan",
            )
            .label(Label::new(ibalik.span().clone()).message("ito ay nasa labas ng paraan"))
            .help("maaari lamang gamitin ang `ibalik` sa loob ng isang paraan");

            return Err(Box::new(diagnostic));
        }

        Ok(())
    }

    fn resolve_expression(&mut self, expression: &mut Expr) -> DiagResult<()> {
        match expression.kind_mut() {
            ExprKind::Integer(_) => Ok(()),
            ExprKind::Float(_) => Ok(()),
            ExprKind::Str { text, interned_id } => {
                let id = self.ctx.intern(text);
                *interned_id = Some(id);

                Ok(())
            }
            ExprKind::Identifier(ident) => match self.lookup_symbol(ident) {
                Some(id) => {
                    expression.set_symbol_id(id);
                    Ok(())
                }
                None => {
                    let current_module = self.current_module();
                    let diagnostic = predefined_diagnostics::use_of_undeclared_variable(
                        current_module,
                        expression.span().clone(),
                    );

                    Err(Box::new(diagnostic))
                }
            },
            ExprKind::Binary { left, right, op } => {
                if let Err(diag) = self.resolve_expression(left) {
                    self.current_module_mut().add_diagnostic(*diag);
                    dbg!(&self.scopes);
                    println!("it errored: {left}")
                }

                if op.kind() == &TokenKind::Equal {
                    self.ensure_valid_assignment(left, op)?;
                }

                self.resolve_expression(right)
            }
            ExprKind::Call { left, args } => {
                if !left.is_lvalue() {
                    let current_module = self.current_module();
                    let diagnostic = TolDiagnostic::err(
                        current_module.source_arc(),
                        current_module.filename(),
                        "pag-tawag ng hindi isang \"lvalue\"",
                    )
                    .label(
                        Label::new(left.span().clone())
                            .message("hindi ito isang \"lvalue\", ngunit tinawag mo ito"),
                    )
                    .help("mga \"lvalue\" lamang ang pwede tawagin");

                    return Err(Box::new(diagnostic));
                }

                self.resolve_expression(left)?;

                for arg in args {
                    if let Err(diag) = self.resolve_expression(arg) {
                        self.current_module_mut().add_diagnostic(*diag);
                    }
                }

                Ok(())
            }
        }
    }

    fn ensure_valid_assignment(&mut self, left: &Expr, op: &Token) -> DiagResult<()> {
        let current_module = self.current_module();
        if !left.is_lvalue() {
            let diagnostic = TolDiagnostic::err(
                current_module.source_arc(),
                current_module.filename(),
                "pag-assign sa hindi \"lvalue\"",
            )
            .label(Label::new(left.span().clone()).message("hindi ito isang \"lvalue\""));

            return Err(Box::new(diagnostic));
        }

        let left_symbol = self.ctx.symbol_by_id(left.symbol_id());
        match left_symbol.kind() {
            SymbolKind::Name { is_mutable, ty } => {
                if !*is_mutable {
                    let diagnostic = TolDiagnostic::err(
                        current_module.source_arc(),
                        current_module.filename(),
                        "pag-iba ng hindi naiibang variable",
                    )
                    .label(
                        Label::new(left_symbol.span().clone())
                            .message("i-dineklara itong hindi naiiba"),
                    )
                    .label(
                        Label::new(left.span().clone())
                            .message("ngunit sinubukan mong ibahin dito"),
                    );

                    Err(Box::new(diagnostic))
                } else {
                    Ok(())
                }
            }
            SymbolKind::Function {
                param_types,
                ret_ty,
            } => {
                let diagnostic = TolDiagnostic::err(
                    current_module.source_arc(),
                    current_module.filename(),
                    "pag-assign sa isang paraan",
                )
                .label(
                    Label::new(left_symbol.span().clone())
                        .message("i-dineklara ito bilaang paraan"),
                )
                .label(Label::new(left.span().clone()).message("sinubukan mong i-assign dito"));

                Err(Box::new(diagnostic))
            }
        }
    }

    fn declare_symbol(&mut self, symbol: Symbol) -> DiagResult<SymbolId> {
        let current_scope = self.scopes.last_mut().unwrap();
        match current_scope.get(symbol.name()) {
            Some(id) => {
                let declared_symbol = self.ctx.symbol_by_id(*id);
                let declared_span = declared_symbol.span().clone();

                let current_module = self.current_module();
                let diagnostic = TolDiagnostic::err(
                    current_module.source_arc(),
                    current_module.filename(),
                    "pag-deklara ng kaparehong pangalan sa iisang sakop",
                )
                .label(Label::new(declared_span).message("na-ideklara na dito"))
                .label(Label::new(symbol.span().clone()).message("dineklara mo ulit dito"));

                Err(Box::new(diagnostic))
            }
            None => {
                let name = symbol.name().to_string();
                let id = self.ctx.add_symbol(symbol);
                current_scope.insert(name, id);

                Ok(id)
            }
        }
    }

    fn lookup_symbol(&mut self, name: &str) -> Option<SymbolId> {
        for scope in self.scopes.iter().rev() {
            if let Some(id) = scope.get(name) {
                return Some(*id);
            }
        }

        None
    }

    fn current_module(&self) -> &Module {
        self.ctx.module_by_id(self.module_id)
    }

    fn current_module_mut(&mut self) -> &mut Module {
        self.ctx.module_by_id_mut(self.module_id)
    }

    fn enter_function(&mut self) {
        self.next_local_slot_stack.push(self.next_local_slot);
        self.next_local_slot = 0;
    }

    fn exit_function(&mut self) -> usize {
        let local_count = self.next_local_slot;
        self.next_local_slot = self
            .next_local_slot_stack
            .pop()
            .expect("function scope stack underflow");

        local_count
    }

    fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn is_in_global_scope(&self) -> bool {
        self.scopes.len() == 1
    }

    fn get_global_slot(&mut self) -> usize {
        let slot = self.next_global_slot;
        self.next_global_slot += 1;

        slot
    }

    fn get_local_slot(&mut self) -> usize {
        let slot = self.next_local_slot;
        self.next_local_slot += 1;

        slot
    }

    fn assign_storage(&mut self) -> Storage {
        if self.is_in_global_scope() {
            Storage::Global(self.get_global_slot())
        } else {
            Storage::Local(self.get_local_slot())
        }
    }
}
