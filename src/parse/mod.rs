//! Handles the lexing and the parsing of the source

use crate::{
    global_ctx::GlobalContext,
    module::{Module, ModuleId},
    parse::ast::{
        expr::{Expr, ExprKind, ListInit},
        stmt::{Branch, Field, Param, ParamList, Stmt, StmtKind},
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

/// Handles the process of turning a bunch of tokens into an Abstract Syntax Tree (AST) representing the language's
/// grammar. See [eng_grammar.txt](./eng_grammar.txt)
pub struct Parser<'c> {
    /// The tokens to be parsed
    tokens: Vec<Token>,

    /// Offset on the vector of tokens pointing to the currently parsed token
    current: usize,

    /// ID pointing to the current module being parsed in the global context
    module_id: usize,

    /// The global context
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

    /// Takes the parser and turns the initialized vector of tokens into an ast (a vector of `Stmt`s)
    /// The ast is stored into the module pointed to by the `module_id`
    pub fn parse(mut self) {
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
            TokenKind::Print => self.parse_print(),
            TokenKind::Paraan => self.parse_paraan(),
            TokenKind::Kung => self.parse_kung(),
            TokenKind::Habang => self.parse_habang(),
            TokenKind::Ibalik => self.parse_ibalik(),
            TokenKind::Klase => self.parse_klase(),
            TokenKind::Biyakin => {
                let span = self.advance().span().clone();
                self.consume(
                    TokenKind::SemiColon,
                    "umaasa ng `;` pagkatapos ng `biyakin`",
                )?;
                Ok(Stmt::new(span, StmtKind::Biyakin))
            }
            TokenKind::Ituloy => {
                let span = self.advance().span().clone();
                self.consume(TokenKind::SemiColon, "umaasa ng `;` pagkatapos ng `ituloy`")?;
                Ok(Stmt::new(span, StmtKind::Ituloy))
            }

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
        let mut is_mutable = false;
        if self.peek().kind() == &TokenKind::Iiba {
            self.advance();
            is_mutable = true;
        }
        let name = self
            .consume_ident("umaasa ako ng pangalan dito para sa pag-deklara")?
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
                is_mutable,
                ty,
                rhs,
            },
        ))
    }

    fn parse_print(&mut self) -> DiagResult<Stmt> {
        let start = self.advance().span().start;
        let expr = self.parse_expression(0)?;
        let end = self
            .consume(
                TokenKind::SemiColon,
                "umaasa ng `;` pagkatapos ng expresyon dito",
            )?
            .span()
            .end;

        Ok(Stmt::new(start..end, StmtKind::Print { expr }))
    }

    fn parse_paraan(&mut self) -> DiagResult<Stmt> {
        let start = self.advance().span().start;
        let name = self
            .consume_ident(
                "umaasa ako ng pangalan dito pagkatapos ng `paraan` para sa pag-deklara",
            )?
            .clone();
        let params = self.parse_params(TokenKind::LParen, TokenKind::RParen)?;
        let ret_ty = if self.peek().kind() == &TokenKind::ThinArrow {
            self.advance();
            dbg!(self.peek().kind());
            self.parse_type()?
        } else {
            TolType::Wala
        };
        self.consume(TokenKind::Colon, "umaasa ako ng `:` dito")?;
        let block = self.parse_block()?;
        let end = block.span().end;

        Ok(Stmt::new(
            start..end,
            StmtKind::Paraan {
                name,
                params,
                ret_ty,
                block: Box::new(block),
            },
        ))
    }

    fn parse_kung(&mut self) -> DiagResult<Stmt> {
        let start = self.advance().span().start;
        let condition = self.parse_expression(0)?;
        self.consume(TokenKind::Colon, "umaasa ng `:` pagkatapos ng kondisyon")?;
        let block = self.parse_block()?;
        let mut end = block.span().end;
        let mut then_branches = vec![Branch::new(Some(condition), block)];

        while !self.at_end() && self.peek().kind() == &TokenKind::Kundi {
            self.advance();
            let condition = self.parse_expression(0)?;
            self.consume(TokenKind::Colon, "umaasa ng `:` pagkatapos ng kondisyon")?;
            let block = self.parse_block()?;
            end = block.span().end;
            then_branches.push(Branch::new(Some(condition), block));
        }

        let mut else_branch = None;
        if self.peek().kind() == &TokenKind::Kungwala {
            self.advance();
            self.consume(TokenKind::Colon, "umaasa ng `:` pagkatapos ng `kungwala`")?;
            let block = self.parse_block()?;
            end = block.span().end;
            else_branch = Some(Box::new(Branch::new(None, block)));
        }

        Ok(Stmt::new(
            start..end,
            StmtKind::Kung {
                then_branches,
                else_branch,
            },
        ))
    }

    fn parse_habang(&mut self) -> DiagResult<Stmt> {
        let start = self.advance().span().start;
        let condition = self.parse_expression(0)?;
        self.consume(TokenKind::Colon, "umaasa ng `:` pagkatapos ng kondisyon")?;
        let block = self.parse_block()?;
        let end = block.span().end;

        Ok(Stmt::new(
            start..end,
            StmtKind::Habang {
                condition,
                block: Box::new(block),
            },
        ))
    }

    fn parse_ibalik(&mut self) -> DiagResult<Stmt> {
        let start = self.advance().span().start;
        let expr = if self.peek().kind() != &TokenKind::SemiColon {
            Some(self.parse_expression(0)?)
        } else {
            None
        };
        let end = self
            .consume(TokenKind::SemiColon, "umaasa ng `;` dito")?
            .span()
            .end;

        Ok(Stmt::new(start..end, StmtKind::Ibalik { expr }))
    }

    fn parse_klase(&mut self) -> DiagResult<Stmt> {
        let start = self.advance().span().start;
        let name = self
            .consume_ident("umaasa ng pangalan ng klase dito")?
            .clone();
        match self.peek().kind() {
            TokenKind::Colon => {
                self.advance();
            }
            TokenKind::SemiColon => {
                let end = self.advance().span().end;
                return Ok(Stmt::new(
                    start..end,
                    StmtKind::Klase {
                        name,
                        methods: Vec::new(),
                    },
                ));
            }
            _ => {
                let current_module = self.current_module();
                let diagnostic = TolDiagnostic::err(
                    current_module.source_arc(),
                    current_module.filename(),
                    "hindi inaasahang token",
                )
                .label(
                    Label::new(self.peek().span().clone()).message("umaasa ako ng `:` o `;` dito"),
                );

                return Err(Box::new(diagnostic));
            }
        }
        self.consume(TokenKind::Indent, "umaasa ng \"indent\" dito")?;
        let mut methods = Vec::new();
        while !self.at_end() && self.peek().kind() != &TokenKind::Dedent {
            match self.parse_paraan() {
                Ok(s) => methods.push(s),
                Err(diag) => self.current_module_mut().add_diagnostic(*diag),
            }
        }

        let end = self
            .consume(TokenKind::Dedent, "umaasa ng \"dedent\" dito")?
            .span()
            .end;

        Ok(Stmt::new(start..end, StmtKind::Klase { name, methods }))
    }

    fn parse_fields(&mut self) -> DiagResult<Vec<Field>> {
        self.consume(TokenKind::Indent, "umaasa ako ng \"indent\" dito")?;

        let mut fields = Vec::new();
        while !self.at_end() && self.peek().kind() != &TokenKind::Dedent {
            let start = self.peek().span().start;
            let is_mutable = if self.peek().kind() == &TokenKind::Iiba {
                self.advance();
                true
            } else {
                false
            };
            let name = self.consume_ident("umaasa ako ng pangalan dito")?.clone();
            let ty = if self.peek().kind() == &TokenKind::Comma {
                self.advance();
                self.parse_type()?
            } else {
                TolType::DiAlam
            };
            let end = self
                .consume(TokenKind::SemiColon, "umaasa ako ng `;` dito")?
                .span()
                .end;

            fields.push(Field {
                name,
                ty,
                span: start..end,
                is_mutable,
            });
        }

        self.consume(TokenKind::Dedent, "umaasa ako ng \"dedent\" dito")?;

        Ok(fields)
    }

    fn parse_block(&mut self) -> DiagResult<Stmt> {
        let start = self
            .consume(TokenKind::Indent, "umaasa ako ng \"indent\"")?
            .span()
            .start;

        let mut statements = Vec::new();
        while !self.at_end() && self.peek().kind() != &TokenKind::Dedent {
            match self.parse_statement() {
                Ok(statement) => statements.push(statement),
                Err(diag) => {
                    self.synchonize();
                    self.current_module_mut().add_diagnostic(*diag);
                }
            }
        }

        let end = self
            .consume(
                TokenKind::Dedent,
                "hindi na-isarado ang sakop na ito gamit ang \"dedent\"",
            )?
            .span()
            .end;

        Ok(Stmt::new(start..end, StmtKind::Block { statements }))
    }

    fn parse_params(
        &mut self,
        begin_delim: TokenKind,
        end_delim: TokenKind,
    ) -> DiagResult<ParamList> {
        let start = self
            .consume(begin_delim, "hindi ko ito inaasahan")?
            .span()
            .start;

        let mut params = Vec::new();
        while !self.at_end() && self.peek().kind() != &end_delim {
            let start = self.peek().span().start;
            let mut is_mutable = false;
            if self.peek().kind() == &TokenKind::Iiba {
                self.advance();
                is_mutable = true;
            }
            let name = self.consume_ident("umaasa ako ng pangalan dito")?.clone();
            let ty = if self.peek().kind() == &TokenKind::Colon {
                self.advance();
                self.parse_type()?
            } else {
                TolType::DiAlam
            };
            let end = self.peek().span().end;

            if self.peek().kind() == &TokenKind::Comma {
                self.advance();
            }

            params.push(Param {
                name,
                ty,
                span: start..end,
                is_mutable,
            });
        }

        let end = self
            .consume(
                end_delim,
                "hindi na-isarado ang mga parametro gamit ang `)`",
            )?
            .span()
            .end;

        Ok(ParamList {
            params,
            span: start..end,
        })
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
            Some(t) => {
                self.advance();
                Ok(t.clone())
            }
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
                Ok(Expr::new(span, ExprKind::Float(x)))
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
            TokenKind::StringLiteral(s) => {
                let s = s.clone();
                let span = self.advance().span().clone();
                Ok(Expr::new(
                    span,
                    ExprKind::Str {
                        text: s,
                        interned_id: None,
                    },
                ))
            }
            TokenKind::Pipe => {
                let start = self.peek().span().start;
                let params = self.parse_params(TokenKind::Pipe, TokenKind::Pipe)?;
                let expr = self.parse_expression(0)?;
                let span = start..expr.span().end;

                Ok(Expr::new(
                    span,
                    ExprKind::AnonymousFn {
                        params,
                        body: Box::new(expr),
                    },
                ))
            }
            TokenKind::At => {
                let start = self.advance().span().start;
                self.consume(TokenKind::LSquare, "umaasa ako ng `[` dito")?;
                let init_expression = self.parse_expression(0)?;
                self.consume(
                    TokenKind::SemiColon,
                    "umaasa ako ng `;` para ihiwalay ang expresyon sa kaliwa at kapasidad sa kanan",
                )?;
                let init_capacity = self
                    .consume_int("umaasa ako ng pangunahing kapasidad na numero")?
                    .clone();
                let end = self
                    .consume(TokenKind::RSquare, "umaasa ako ng `]` dito")?
                    .span()
                    .end;
                let span = start..end;

                Ok(Expr::new(
                    span,
                    ExprKind::List {
                        elements: Vec::new(),
                        init: Some(ListInit {
                            expr: Box::new(init_expression),
                            init_capacity,
                        }),
                    },
                ))
            }
            TokenKind::LSquare => {
                let start = self.peek().span().start;
                self.consume(TokenKind::LSquare, "umaasa ako ng `[` dito")?;
                let elements = self.parse_many_expressions(TokenKind::RSquare)?;
                let end = self
                    .consume(TokenKind::RSquare, "umaasa ako ng `]` dito")?
                    .span()
                    .end;
                let span = start..end;

                Ok(Expr::new(
                    span,
                    ExprKind::List {
                        elements,
                        init: None,
                    },
                ))
            }
            _ => {
                let current_module = self.current_module();
                let span = self.peek().span().clone();
                let diagnostic = predefined_diagnostics::unexpected_token(
                    current_module,
                    "maling pagsimula ng expresyon",
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
            TokenKind::Plus
            | TokenKind::PlusEq
            | TokenKind::PlusPlus
            | TokenKind::Minus
            | TokenKind::MinusEq
            | TokenKind::Star
            | TokenKind::StarEq
            | TokenKind::Slash
            | TokenKind::SlashEq
            | TokenKind::Percent
            | TokenKind::PercentEq
            | TokenKind::Equal
            | TokenKind::EqualEq
            | TokenKind::NotEq
            | TokenKind::Greater
            | TokenKind::GreatEq
            | TokenKind::Lesser
            | TokenKind::LessEq => {
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
            TokenKind::LParen => {
                let args = self.parse_many_expressions(TokenKind::RParen)?;
                let end = self.consume(TokenKind::RParen, ")")?.span().end;
                let span = left.span().start..end;

                Ok(Expr::new(
                    span,
                    ExprKind::Call {
                        left: Box::new(left),
                        args,
                    },
                ))
            }
            TokenKind::Dot => {
                let field = self
                    .consume_ident("umaasa ako ng pangalan dito pagkatapos ng `.`")?
                    .clone();
                let span = left.span().start..field.span().end;

                Ok(Expr::new(
                    span,
                    ExprKind::FieldAccess {
                        object: Box::new(left),
                        field,
                    },
                ))
            }
            TokenKind::LSquare => {
                let index = self.parse_expression(0)?;
                let end = self
                    .consume(TokenKind::RSquare, "umaasa ako ng `]` dito")?
                    .span()
                    .end;
                let span = left.span().start..end;

                Ok(Expr::new(
                    span,
                    ExprKind::IndexAccess {
                        left: Box::new(left),
                        index: Box::new(index),
                    },
                ))
            }
            _ => unreachable!(),
        }
    }

    // Parse multiple expressions until a given delimiter
    fn parse_many_expressions(&mut self, end_delim: TokenKind) -> DiagResult<Vec<Expr>> {
        let mut exprs = Vec::new();
        while !self.at_end() && self.peek().kind() != &end_delim {
            exprs.push(self.parse_expression(0)?);

            if self.peek().kind() == &TokenKind::Comma {
                self.advance();
            }
        }

        Ok(exprs)
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

    fn peek_next(&self) -> &Token {
        &self.tokens[self.current + 1]
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

    fn consume_int(&mut self, message: impl Into<String>) -> DiagResult<&Token> {
        if matches!(self.peek().kind(), TokenKind::IntLiteral(_)) {
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
