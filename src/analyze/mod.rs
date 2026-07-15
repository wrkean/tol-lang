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
        token::TokenKind,
    },
};

pub mod symbol;

type Scope = HashMap<String, SymbolId>;

/// Analyzes the target module's ast
pub struct Analyzer<'gctx> {
    scopes: Vec<Scope>,
    ctx: &'gctx mut GlobalContext,
    module_id: ModuleId,

    next_global_slot: usize,
    next_local_slot: usize,
}

impl<'gctx> Analyzer<'gctx> {
    /// Creates a new analyze instance that targets the given module by id
    pub fn new(ctx: &'gctx mut GlobalContext, module_id: ModuleId) -> Self {
        Self {
            scopes: vec![Scope::new()],
            ctx,
            module_id,
            next_global_slot: 0,
            next_local_slot: 0,
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
            StmtKind::Print { expr } => self.resolve_expression(expr),
            StmtKind::Expr { expr } => self.resolve_expression(expr),
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

    fn resolve_expression(&mut self, expression: &mut Expr) -> DiagResult<()> {
        match expression.kind_mut() {
            ExprKind::Integer(_) => Ok(()),
            ExprKind::Float(_) => Ok(()),
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
                }

                self.resolve_expression(right)
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
