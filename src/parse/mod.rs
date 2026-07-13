//! Handles the lexing and the parsing of the source

use crate::{
    compiler::{Compiler, Module, ModuleId},
    parse::ast::expr::{Expr, ExprKind},
    prelude::DiagResult,
    tol::{
        diagnostic::{self, Label, TolDiagnostic, predefined_diagnostics},
        operator,
        token::{Token, TokenKind},
    },
};

pub mod ast;
pub mod lexer;

/// Handles turning a bunch of tokens into an Abstract Syntax Tree (AST) representing the language's
/// grammar
///
/// see [eng_grammar.txt](./eng_grammar.txt)
pub struct Parser<'c> {
    tokens: Vec<Token>,
    current: usize,
    module_id: usize,
    compiler: &'c mut Compiler,
}

impl<'c> Parser<'c> {
    pub fn new(tokens: Vec<Token>, compiler: &'c mut Compiler, module_id: ModuleId) -> Self {
        Self {
            tokens,
            current: 0,
            module_id,
            compiler,
        }
    }

    /// Runs the parser from the initialized vector of tokens and generates an ast (a vector of `Stmt`s)
    pub fn parse(&mut self) -> Expr {
        self.parse_expression(0).unwrap()
    }

    fn parse_expression(&mut self, precedence: u8) -> DiagResult<Expr> {
        let mut left = self.nud()?;

        while !self.at_end() && operator::precedence(self.peek().kind()) > precedence {
            let op = self.advance().clone();
            left = self.led(op, left)?;
        }

        Ok(left)
    }

    fn nud(&mut self) -> DiagResult<Expr> {
        match self.peek().kind() {
            TokenKind::IntLiteral(x) => {
                let x = *x;
                let span = self.advance().span().clone(); // Advanced here
                Ok(Expr::new(span, ExprKind::Integer(x)))
            }
            TokenKind::FloatLiteral(x) => {
                let x = *x;
                let span = self.advance().span().clone();
                Ok(Expr::new(span, ExprKind::FloatLiteral(x)))
            }
            TokenKind::LParen => {
                let start = self.advance(); // Consume `(`
                let expr = self.parse_expression(0)?;
                self.consume(
                    TokenKind::RParen,
                    "hindi na-isarado ang `)` pagkatapos ng expresyon",
                )?;

                Ok(expr)
            }
            _ => {
                let current_module = self.current_module();
                let span = self.peek().span().clone();
                let diagnostic = predefined_diagnostics::unexpected_token(
                    current_module,
                    "hindi ito maaaring mag-simula ng isang expresyon",
                    span,
                );

                Err(Box::new(diagnostic))
            }
        }
    }

    fn led(&mut self, op: Token, left: Expr) -> DiagResult<Expr> {
        let op_kind = op.kind();
        let precedence = match operator::associativity(op_kind) {
            operator::Associativity::Left => operator::precedence(op_kind) + 1,
            operator::Associativity::Right => operator::precedence(op_kind),
        };

        match op_kind {
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash => {
                let right = self.parse_expression(0)?;
                let span = left.span().start..right.span().end;

                Ok(Expr::new(
                    span,
                    ExprKind::Binary {
                        left: Box::new(left),
                        right: Box::new(right),
                        op,
                    },
                ))
            }
            _ => unreachable!(),
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> &Token {
        self.current += 1;

        &self.tokens[self.current - 1]
    }

    fn consume(
        &mut self,
        expected_kind: TokenKind,
        message: impl Into<String>,
    ) -> DiagResult<&Token> {
        if self.peek().kind() == &expected_kind {
            return Ok(self.advance());
        }

        let current_module = self.current_module();
        let span = self.peek().span().clone();

        let diagnostic =
            predefined_diagnostics::unexpected_token(current_module, message.into(), span);

        Err(Box::new(diagnostic))
    }

    fn current_module(&self) -> &Module {
        self.compiler.module_by_id(self.module_id)
    }

    fn at_end(&self) -> bool {
        self.peek().kind() == &TokenKind::Eof
    }
}
