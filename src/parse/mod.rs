//! Handles the lexing and the parsing of the source

use crate::{
    global_ctx::GlobalContext,
    module::{Module, ModuleId},
    parse::ast::{
        expr::{Expr, ExprKind},
        stmt::{Stmt, StmtKind},
    },
    prelude::DiagResult,
    tol::{
        diagnostic::{self, Label, TolDiagnostic, predefined_diagnostics},
        operator,
        token::{Token, TokenKind},
        types::{TYPES, TolType},
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
    ctx: &'c mut GlobalContext,
}

impl<'c> Parser<'c> {
    /// Create a new parser that targets the given module by id
    pub fn new(tokens: Vec<Token>, ctx: &'c mut GlobalContext, module_id: ModuleId) -> Self {
        Self {
            tokens,
            current: 0,
            module_id,
            ctx,
        }
    }

    /// Runs the parser from the initialized vector of tokens and generates an ast (a vector of `Stmt`s) for the target module
    pub fn parse(&mut self) {
        while !self.at_end() {
            match self.parse_statement() {
                Ok(statement) => {
                    let current_module = self.current_module_mut();
                    current_module.add_statement(statement);
                }
                Err(diag) => {
                    let current_module = self.current_module_mut();
                    current_module.add_diagnostic(*diag);
                    self.synchonize();
                }
            }
        }
    }

    fn parse_statement(&mut self) -> DiagResult<Stmt> {
        match self.peek().kind() {
            TokenKind::Ang => self.parse_ang(),

            _ => {
                let expr = self.parse_expression(0)?;
                let end = self
                    .consume(
                        TokenKind::SemiColon,
                        "umaasa ako ng `;` pagkatapos ng expresyon dito",
                    )?
                    .span()
                    .end;
                let span = expr.span().start..end;

                Ok(Stmt::new(span, StmtKind::Expr { expr }))
            }
        }
    }

    fn parse_ang(&mut self) -> DiagResult<Stmt> {
        let start = self.advance().span().start;
        let name = self
            .consume_ident("umaasa ako ng variable dito para sa pag-deklara")?
            .clone();
        let ty = match self.peek().kind() {
            TokenKind::Colon => {
                self.advance();
                self.parse_type()?
            }
            _ => TolType::DiAlam,
        };
        self.consume(TokenKind::Equal, "umaasa ako ng `=` dito")?;
        let rhs = self.parse_expression(0)?;
        let end = self
            .consume(
                TokenKind::SemiColon,
                "umaasa ako ng `;` pagkatapos ng expresyon dito",
            )?
            .span()
            .end;

        Ok(Stmt::new(
            start..end,
            StmtKind::Ang {
                name,
                is_mutable: true,
                ty,
                rhs,
            },
        ))
    }

    fn parse_type(&mut self) -> DiagResult<TolType> {
        let TokenKind::Identifier(ty_str) = self.peek().kind() else {
            let diagnostic = predefined_diagnostics::unexpected_token(
                self.current_module(),
                "umaasa ako ng tipo dito",
                self.peek().span().clone(),
            );

            return Err(Box::new(diagnostic));
        };

        match TYPES.get(ty_str) {
            Some(t) => Ok(t.clone()),
            None => {
                let diagnostic = predefined_diagnostics::unrecognized_type(
                    self.current_module(),
                    self.peek().span().clone(),
                );
                Err(Box::new(diagnostic))
            }
        }
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
            TokenKind::Identifier(s) => {
                let s = s.clone();
                let span = self.advance().span().clone();
                Ok(Expr::new(span, ExprKind::Identifier(s)))
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

    fn synchonize(&mut self) {
        if self.at_end() {
            return;
        }

        self.advance();

        while !self.at_end() {
            let previous = &self.tokens[self.current - 1];

            // Tokens provided here are statement delimiters
            if matches!(previous.kind(), TokenKind::SemiColon | TokenKind::RBrace) {
                return;
            }

            if self.peek().kind().is_synchronization_point() {
                return;
            }

            self.advance();
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

    fn consume_ident(&mut self, message: impl Into<String>) -> DiagResult<&Token> {
        if matches!(self.peek().kind(), TokenKind::Identifier(_)) {
            return Ok(self.advance());
        }

        let current_module = self.current_module();
        let span = self.peek().span().clone();

        let diagnostic =
            predefined_diagnostics::unexpected_token(current_module, message.into(), span);

        Err(Box::new(diagnostic))
    }

    fn current_module(&self) -> &Module {
        self.ctx.module_by_id(self.module_id)
    }

    fn current_module_mut(&mut self) -> &mut Module {
        self.ctx.module_by_id_mut(self.module_id)
    }

    fn at_end(&self) -> bool {
        self.peek().kind() == &TokenKind::Eof
    }
}
