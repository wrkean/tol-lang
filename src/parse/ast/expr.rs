use std::{collections::HashMap, fmt};

use crate::{
    analyze::{ResolvedVar, symbol::SymbolId},
    parse::ast::stmt::{ParamList, Stmt},
    tol::token::{Span, Token},
};

/// Ast node representing expressions
pub struct Expr {
    /// Span pointing to the source this expression is parsed from. The node itself DOES NOT know
    /// which source it points to, it is handled by the compilation pipeline
    span: Span,

    /// The kind of expression
    kind: ExprKind,

    /// Resolved variable, available only after name resolution. Some expressions may not have a
    /// resolved var, typically those expressions that refer to no names like the integer
    /// expression
    resolved_var: Option<ResolvedVar>,
}

impl Expr {
    /// Creates a new expression
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

    /// L-values are expressions that appear to the left of `=` in an assignment expression or
    /// name declarations. This function returns true if this expression is indeed an l-value
    pub fn is_lvalue(&self) -> bool {
        use ExprKind::*;
        matches!(
            self.kind(),
            Identifier(_) | AnonymousFn { .. } | FieldAccess { .. } | IndexAccess { .. }
        )
    }
}

/// Enum representing different kinds of expressions
pub enum ExprKind {
    /// Integer literals
    ///
    /// e.g.: `1`
    Integer(i64),

    /// Float literals
    ///
    /// e.g.: `4.2`
    Float(f64),

    /// Identifiers. Names in the source code that are not keywords basically.
    Identifier(String),

    /// Anonymous function
    ///
    /// e.g.: `|param1| param1 + 1`
    AnonymousFn { params: ParamList, body: Box<Expr> },

    /// String literals
    ///
    /// e.g.: `"Hello World!"`
    Str {
        text: String,
        interned_id: Option<usize>,
    },

    /// Binary expressions
    ///
    /// e.g.: `4 + 2`
    Binary {
        left: Box<Expr>,
        right: Box<Expr>,
        op: Token,
    },

    // TODO: Unary expressions
    // Unary {
    //      operand: Box<Expr>,
    //      op: Token,
    // }
    //
    /// Function call expressions
    ///
    /// e.g.: `foo()`
    Call { left: Box<Expr>, args: Vec<Expr> },

    /// Field access expressions
    ///
    /// e.g: `class_inst1.foo`
    FieldAccess { object: Box<Expr>, field: Token },

    /// List literals
    ///
    /// e.g. `[0, 1, 2, 3]` or `@[expr1; 100]`
    List {
        elements: Vec<Expr>,
        init: Option<ListInit>,
    },

    /// Index access expression
    ///
    /// e.g.: `list1[index]`
    IndexAccess { left: Box<Expr>, index: Box<Expr> },
}

/// The list initializer. During parsing, if `@` is encountered before a string literal, the parser,
/// instead of parsing a list of expressions, parses a list initializer syntax instead:
/// `@[<expr>: <count>]`
pub struct ListInit {
    /// The expression to initialize
    pub expr: Box<Expr>,

    /// The capacity, filled with the expression given
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
