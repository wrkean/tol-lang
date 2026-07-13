use crate::{
    parse::ast::expr::Expr,
    tol::{token::Span, types::TolType},
};

/// Ast node representing statements
pub struct Stmt {
    span: Span,
    kind: StmtKind,
}

impl Stmt {
    pub fn new(span: Span, kind: StmtKind) -> Self {
        Self { span, kind }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn kind(&self) -> &StmtKind {
        &self.kind
    }
}

pub enum StmtKind {
    // Name declaration
    Ang { is_mutable: bool, ty: TolType },

    // Expression statement
    Expr { expr: Expr },
}
