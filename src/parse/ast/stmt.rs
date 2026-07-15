use crate::{
    analyze::symbol::SymbolId,
    parse::ast::expr::Expr,
    prelude::Spanned,
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
    Print {
        expr: Expr,
    },
    Paraan {
        name: Token,
        params: ParamList,
        ret_ty: TolType,
        block: Box<Stmt>,
    },
    Block {
        statements: Vec<Stmt>,
    },
    Kung {
        then_branches: Vec<Branch>,
        else_branch: Option<Box<Branch>>,
    },

    // Expression statement
    Expr {
        expr: Expr,
    },
}

pub struct ParamList {
    pub params: Vec<Param>,
    pub span: Span,
}

impl ParamList {
    pub fn spanned_types(&self) -> Spanned<Vec<TolType>> {
        let param_types: Vec<TolType> = self.params.iter().map(|param| param.ty.clone()).collect();

        Spanned::new(self.span.clone(), param_types)
    }

    pub fn len(&self) -> usize {
        self.params.len()
    }
}

pub struct Param {
    pub name: Token,
    pub ty: TolType,
    pub span: Span,
    pub is_mutable: bool,
}

pub struct Branch {
    pub condition: Option<Expr>,
    pub block: Stmt,
}

impl Branch {
    pub fn new(condition: Option<Expr>, block: Stmt) -> Self {
        Self { condition, block }
    }
}
