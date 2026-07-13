use crate::parse::ast::stmt::Stmt;

pub mod expr;
pub mod stmt;

pub type Ast = Vec<Stmt>;
