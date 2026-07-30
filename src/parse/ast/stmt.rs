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
    /// Span pointing to the source this statement is parsed from. The node itself DOES NOT know
    /// which source it points to, it is handled by the compilation pipeline
    span: Span,

    /// The kind of statement
    kind: StmtKind,

    /// Resolved variable, available only after name resolution. Some statements may not have a
    /// resolved var, typically those statements with no names attached to them like the expression
    /// statement
    resolved_var: Option<ResolvedVar>,
}

impl Stmt {
    /// Creates a new statement node
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

/// Enum representing different kinds of statements
pub enum StmtKind {
    /// Name declaration statement.
    ///
    /// e.g.: `ang x = 5`
    Ang { name: Token, ty: TolType, rhs: Expr },

    /// Function declaration statement
    ///
    /// e.g.:
    /// ```
    /// paraan foo(arg1, arg2):
    ///     ang x = 5
    /// ```
    Paraan {
        name: Token,
        params: ParamList,
        ret_ty: TolType,
        block: Box<Stmt>,
    },

    /// A block statement, contains zero or more statements
    Block { statements: Vec<Stmt> },

    /// If-statement
    ///
    /// e.g.:
    /// ```
    /// kung condition1:
    ///     x += 1
    /// kundi condition2:
    ///     x -= 1
    /// kungwala:
    ///     x = 0
    /// ```
    Kung {
        then_branches: Vec<Branch>,
        else_branch: Option<Box<Branch>>,
    },

    /// While-statement
    ///
    /// e.g.:
    /// ```
    /// habang condition1:
    ///     x += 1
    /// ```
    Habang { condition: Expr, block: Box<Stmt> },

    /// Return statement
    ///
    /// e.g.: `ibalik 0`
    Ibalik { expr: Option<Expr> },

    /// Class declaration statement
    ///
    /// e.g.:
    /// ```
    /// klase Class1:
    ///     paraan bago():
    ///         ibalik Class1()
    /// ```
    Klase { name: Token, methods: Vec<Stmt> },

    /// Expression statement. An expression with a semicolon at the end. Boring.
    Expr { expr: Expr },

    /// Import statement
    ///
    /// e.g.: `kunin "/std/io"`
    Kunin {
        segments: Vec<Token>,
        import_path_type: ImportPathType,
    },

    /// Break statement
    Biyakin,

    /// Continue statement
    Ituloy,
}

/// The function parameter list
pub struct ParamList {
    /// List of params
    pub params: Vec<Param>,

    /// The span from the first parameter up to the last parameter
    pub span: Span,
}

impl ParamList {
    /// Turns the vector of parameters into a list of types with a span
    pub fn spanned_types(&self) -> Spanned<Vec<TolType>> {
        let param_types: Vec<TolType> = self.params.iter().map(|param| param.ty.clone()).collect();

        Spanned::new(self.span.clone(), param_types)
    }

    pub fn len(&self) -> usize {
        self.params.len()
    }
}

/// The function parameter
#[derive(Clone)]
pub struct Param {
    /// Name of the parameter
    pub name: Token,

    /// The type of the parameter, may be omitted by the user, which makes this `TolType::DiAlam`
    /// (or Unknown in english)
    pub ty: TolType,

    /// Span of the parameter, from its name, to its type (if given)
    pub span: Span,
}

pub type Field = Param;

/// The conditional branch
pub struct Branch {
    /// Condition of the branch.
    ///
    /// Is `Some(cond)` if it is parsed after `kung` and `kundi` keywords, `None` if it is parsed
    /// after the `kungwala` keyword. Picture and if-elif-else in python where conditions are given
    /// at the former keywords but not at the latter
    pub condition: Option<Expr>,

    /// The block of the branch
    pub block: Stmt,
}

impl Branch {
    /// Creates a new branch
    pub fn new(condition: Option<Expr>, block: Stmt) -> Self {
        Self { condition, block }
    }
}

/// The import path type determine where the analyzer starts to search for the path
/// ```
/// Std: "/std/io" -> searches $TOL_STD_PATH/io.tol
/// Relative: "./file1" -> searches ./file1.tol
/// Root: "folder1/file1" -> searches $TOL_PROJECT_ROOT/folder1/file1.tol
pub enum ImportPathType {
    /// Searches the std path
    Std,

    /// Searches the relative path
    Relative,

    /// Searches the package root path.
    ///
    /// TODO: Implement later
    Root,
}
