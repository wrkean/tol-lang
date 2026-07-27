use std::{collections::HashMap, fmt};

use crate::{
    analyze::{ResolvedVar, symbol::SymbolId},
    parse::ast::stmt::{ParamList, Stmt},
    tol::token::{Span, Token},
};

/// Ast node representing expressions
pub struct Expr {
    span: Span,
    kind: ExprKind,
    resolved_var: Option<ResolvedVar>,
}

impl Expr {
    pub fn new(span: Span, kind: ExprKind) -> Self {
        Self {
            span,
            kind,
            resolved_var: None,
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

    pub fn set_resolved_var(&mut self, var: ResolvedVar) {
        self.resolved_var = Some(var);
    }

    pub fn resolved_var(&self) -> ResolvedVar {
        self.resolved_var
            .clone()
            .expect("this expression's symbol id is never set")
    }

    pub fn is_lvalue(&self) -> bool {
        use ExprKind::*;
        matches!(
            self.kind(),
            Identifier(_) | AnonymousFn { .. } | FieldAccess { .. } | IndexAccess { .. }
        )
    }
}

/// The kind of expression AST node. Should be owned by `Expr`
pub enum ExprKind {
    Integer(i64),
    Float(f64),
    Identifier(String),
    AnonymousFn {
        params: ParamList,
        body: Box<Expr>,
    },
    Str {
        text: String,
        interned_id: Option<usize>,
    },
    Binary {
        left: Box<Expr>,
        right: Box<Expr>,
        op: Token,
    },
    Call {
        left: Box<Expr>,
        args: Vec<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: Token,
    },
    List {
        elements: Vec<Expr>,
        init: Option<ListInit>,
    },
    IndexAccess {
        left: Box<Expr>,
        index: Token,
    },
}

pub struct ListInit {
    pub expr: Box<Expr>,
    pub init_capacity: Token,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ExprKind::Integer(x) => write!(f, "{x}"),
            ExprKind::Float(x) => write!(f, "{x}"),
            ExprKind::Identifier(s) => write!(f, "{s}"),
            ExprKind::Str { text, interned_id } => write!(f, "{text}"),
            ExprKind::Binary { left, right, op } => {
                write!(f, "({:?} {} {})", op.kind(), left, right)
            }
            ExprKind::AnonymousFn { .. } => unimplemented!(),
            ExprKind::Call { .. } => unimplemented!(),
            ExprKind::FieldAccess { .. } => unimplemented!(),
            ExprKind::List { .. } => unimplemented!(),
            ExprKind::IndexAccess { .. } => unimplemented!(),
        }
    }
}
