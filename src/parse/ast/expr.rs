use std::fmt;

use crate::{
    analyze::symbol::SymbolId,
    tol::token::{Span, Token},
};

/// Ast node representing expressions
pub struct Expr {
    span: Span,
    kind: ExprKind,
    symbol_id: Option<SymbolId>,
}

impl Expr {
    pub fn new(span: Span, kind: ExprKind) -> Self {
        Self {
            span,
            kind,
            symbol_id: None,
        }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn kind(&self) -> &ExprKind {
        &self.kind
    }

    pub fn kind_mut(&mut self) -> &mut ExprKind {
        &mut self.kind
    }

    pub fn set_symbol_id(&mut self, id: SymbolId) {
        self.symbol_id = Some(id);
    }

    pub fn symbol_id(&self) -> usize {
        self.symbol_id
            .expect("this expression's symbol id is never set")
    }

    pub fn is_lvalue(&self) -> bool {
        use ExprKind::*;
        matches!(self.kind(), Identifier(_))
    }
}

/// The kind of expression AST node. Should be owned by `Expr`
pub enum ExprKind {
    Integer(i64),
    Float(f64),
    Identifier(String),
    Binary {
        left: Box<Expr>,
        right: Box<Expr>,
        op: Token,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ExprKind::Integer(x) => write!(f, "{x}"),
            ExprKind::Float(x) => write!(f, "{x}"),
            ExprKind::Identifier(s) => write!(f, "{s}"),
            ExprKind::Binary { left, right, op } => {
                write!(f, "({:?} {} {})", op.kind(), left, right)
            }
        }
    }
}
