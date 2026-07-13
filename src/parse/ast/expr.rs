use std::fmt;

use crate::tol::token::{Span, Token};

pub struct Expr {
    span: Span,
    kind: ExprKind,
}

impl Expr {
    pub fn new(span: Span, kind: ExprKind) -> Self {
        Self { span, kind }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}

pub enum ExprKind {
    Integer(i64),
    FloatLiteral(f64),
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
            ExprKind::FloatLiteral(x) => write!(f, "{x}"),
            ExprKind::Binary { left, right, op } => {
                write!(f, "({:?} {} {})", op.kind(), left, right)
            }
        }
    }
}
