use crate::{
    analyze::{ResolvedVar, symbol::SymbolId},
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
    resolved_var: Option<ResolvedVar>,
}

impl Stmt {
    pub fn new(span: Span, kind: StmtKind) -> Self {
        Self {
            span,
            kind,
            resolved_var: None,
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

    pub fn set_resolved_var(&mut self, var: ResolvedVar) {
        self.resolved_var = Some(var);
    }

    pub fn resolved_var(&self) -> ResolvedVar {
        self.resolved_var
            .clone()
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
    Habang {
        condition: Expr,
        block: Box<Stmt>,
    },
    Ibalik {
        expr: Option<Expr>,
    },
    Klase {
        name: Token,
        fields: Vec<Field>,
    },

    // Expression statement
    Expr {
        expr: Expr,
    },

    Biyakin,
    Ituloy,
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

#[derive(Clone)]
pub struct Param {
    pub name: Token,
    pub ty: TolType,
    pub span: Span,
    pub is_mutable: bool,
}

pub type Field = Param;

pub struct Branch {
    pub condition: Option<Expr>,
    pub block: Stmt,
}

impl Branch {
    pub fn new(condition: Option<Expr>, block: Stmt) -> Self {
        Self { condition, block }
    }
}
