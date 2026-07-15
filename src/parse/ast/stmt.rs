use crate::{
    analyze::symbol::SymbolId,
    parse::ast::expr::Expr,
    tol::{
        token::{Span, Token},
        types::TolType,
    },
};

/// Ast node representing statements
pub struct Stmt {
    span: Span,
    kind: StmtKind,
    symbol_id: Option<SymbolId>,
}

impl Stmt {
    pub fn new(span: Span, kind: StmtKind) -> Self {
        Self {
            span,
            kind,
            symbol_id: None,
        }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn kind(&self) -> &StmtKind {
        &self.kind
    }

    pub fn kind_mut(&mut self) -> &mut StmtKind {
        &mut self.kind
    }

    pub fn set_symbol_id(&mut self, id: SymbolId) {
        self.symbol_id = Some(id);
    }

    pub fn symbol_id(&self) -> usize {
        self.symbol_id
            .expect("this statement's symbol id is never set")
    }
}

pub enum StmtKind {
    // Name declaration
    Ang {
        name: Token,
        is_mutable: bool,
        ty: TolType,
        rhs: Expr,
    },

    // Expression statement
    Expr {
        expr: Expr,
    },
}
