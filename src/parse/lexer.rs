use std::{cell::RefCell, iter::Peekable, mem, rc::Rc, str::Chars, sync::Arc};

use crate::{
    global_ctx::GlobalContext,
    module::{Module, ModuleId},
    tol::{
        diagnostic::{Label, TolDiagnostic},
        keywords::KEYWORDS,
        token::{Span, Token, TokenKind},
    },
};

/// Responsible for turning source to a vector of tokens
pub struct Lexer<'src, 'gctx> {
    source: &'src str,
    source_iter: Peekable<Chars<'src>>,
    start: usize,
    current: usize,
    bracket_depth: u8,
    tokens: Vec<Token>,
    indent_stack: Vec<u8>,
    at_line_start: bool,
    module_id: ModuleId,
    ctx: &'gctx mut GlobalContext,
}

impl<'src, 'gctx> Lexer<'src, 'gctx> {
    /// Creates a new lexer instance, to lex the given source
    pub fn new(source: &'src str, ctx: &'gctx mut GlobalContext, module_id: ModuleId) -> Self {
        Self {
            source,
            source_iter: source.chars().peekable(),
            start: 0,
            current: 0,
            bracket_depth: 0,
            tokens: Vec::new(),
            indent_stack: vec![0],
            at_line_start: true,
            ctx,
            module_id,
        }
    }

    /// Lexes the given source string and returns a list of tokens
    pub fn lex(&mut self) -> Vec<Token> {
        while let Some(current_char) = self.peek() {
            self.start = self.current;

            if self.at_line_start {
                self.handle_indents();
                continue;
            }

            self.lex_token(current_char);
        }

        if self
            .tokens
            .last()
            .is_some_and(|tok| tok.kind().infers_semicolon())
        {
            self.add_token(TokenKind::SemiColon, self.current_span());
        }

        self.emit_remaining_dedents();

        self.add_token(TokenKind::Eof, self.current_span());

        mem::take(&mut self.tokens)
    }

    fn lex_token(&mut self, current_char: char) {
        self.advance();

        match current_char {
            '(' | '[' => {
                self.bracket_depth += 1;
                match current_char {
                    '(' => self.add_token(TokenKind::LParen, self.current_span()),
                    '[' => self.add_token(TokenKind::LSquare, self.current_span()),
                    _ => unreachable!(),
                };
            }
            ')' | ']' => {
                self.bracket_depth -= 1;
                match current_char {
                    ')' => self.add_token(TokenKind::RParen, self.current_span()),
                    ']' => self.add_token(TokenKind::RSquare, self.current_span()),
                    _ => unreachable!(),
                };
            }
            '{' => self.add_token(TokenKind::LBrace, self.current_span()),
            '}' => self.add_token(TokenKind::RBrace, self.current_span()),
            ':' => self.add_token(TokenKind::Colon, self.current_span()),
            ';' => self.add_token(TokenKind::SemiColon, self.current_span()),
            '!' => {
                if self.match_ch('=') {
                    self.add_token(TokenKind::NotEq, self.current_span())
                } else {
                    todo!()
                }
            }
            '=' => {
                if self.match_ch('=') {
                    self.add_token(TokenKind::EqualEq, self.current_span());
                } else {
                    self.add_token(TokenKind::Equal, self.current_span());
                }
            }
            '>' => {
                if self.match_ch('=') {
                    self.add_token(TokenKind::GreatEq, self.current_span());
                } else {
                    self.add_token(TokenKind::Greater, self.current_span());
                }
            }
            '<' => {
                if self.match_ch('=') {
                    self.add_token(TokenKind::LessEq, self.current_span());
                } else {
                    self.add_token(TokenKind::Lesser, self.current_span());
                }
            }
            '+' => self.add_token(TokenKind::Plus, self.current_span()),
            '-' => {
                if self.match_ch('>') {
                    self.add_token(TokenKind::ThinArrow, self.current_span());
                } else {
                    self.add_token(TokenKind::Minus, self.current_span())
                }
            }
            '*' => self.add_token(TokenKind::Star, self.current_span()),
            '/' => self.add_token(TokenKind::Slash, self.current_span()),
            ',' => self.add_token(TokenKind::Comma, self.current_span()),
            '\n' => {
                if self.bracket_depth == 0
                    && self
                        .tokens
                        .last()
                        .is_some_and(|tok| tok.kind().infers_semicolon())
                {
                    self.emit_inferred_semicolon();
                }
                self.at_line_start = true;
            }
            ' ' | '\t' | '\r' => { /* skip irrelevant whitespace */ }
            ch if ch.is_ascii_alphabetic() || ch == '_' => self.lex_identifier(),
            ch if ch.is_ascii_digit() => self.lex_number(),
            _ => {
                let current_module = self.current_module();
                let diagnostic = TolDiagnostic::err(
                    current_module.source_arc(),
                    current_module.filename(),
                    "paggamit ng karakter na hindi parte ng sintax",
                )
                .label(
                    Label::new(self.current_span())
                        .message("ang karakter na ito ay hindi parte ng sintax ng tol"),
                )
                .help("subukan mo itong tanggalin");
                self.current_module_mut().add_diagnostic(diagnostic);
            }
        }
    }

    fn handle_indents(&mut self) {
        let mut current_indent = 0;
        while let Some(ch) = self.peek() {
            if ch == ' ' {
                current_indent += 1;
            } else if ch == '\t' {
                current_indent += 4;
            } else {
                break;
            }
            self.advance();
        }
        if matches!(self.peek(), Some('\n') | None) {
            self.at_line_start = false;
            if self.peek() == Some('\n') {
                self.advance();
            }
            self.at_line_start = true;
            return;
        }
        self.at_line_start = false;
        if current_indent > *self.indent_stack.last().unwrap() {
            let last_was_colon = self
                .tokens
                .last()
                .is_some_and(|t| t.kind() == &TokenKind::Colon);

            if !last_was_colon {
                let current_module = self.current_module();
                let diagnostic = TolDiagnostic::err(
                    current_module.source_arc(),
                    current_module.filename(),
                    "hindi inaasahang pag-\"indent\"",
                )
                .label(
                    Label::new(self.current_span()).message("hindi ka dapat mag-\"indent\" dito"),
                )
                .help("sa tol, maaari lamang mag-\"indent\" pagkatapos ng isang `:`");
                self.current_module_mut().add_diagnostic(diagnostic);
            } else {
                self.indent_stack.push(current_indent);
                self.add_token(TokenKind::Indent, self.current_span());
            }
        } else if current_indent < *self.indent_stack.last().unwrap() {
            while current_indent < *self.indent_stack.last().unwrap() {
                self.indent_stack.pop();
                self.add_token(TokenKind::Dedent, self.current_span());
            }
            if current_indent != *self.indent_stack.last().unwrap() {
                let current_module = self.current_module();
                let levels = self
                    .indent_stack
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                let diagnostic = TolDiagnostic::err(
                    current_module.source_arc(),
                    current_module.filename(),
                    "hindi tumutugma ang antas ng indentasyon",
                )
                .label(Label::new(self.current_span()).message(format!(
                    "{current_indent} na espasyo, pero ang mga bukas na antas ay: {levels}"
                )));
                self.current_module_mut().add_diagnostic(diagnostic);
            }
        }
    }

    fn emit_remaining_dedents(&mut self) {
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            self.add_token(TokenKind::Dedent, self.current_span());
        }
    }

    fn add_token(&mut self, kind: TokenKind, span: Span) {
        let token = Token::new(span, kind);

        self.tokens.push(token);
    }

    fn lex_identifier(&mut self) {
        while matches!(self.peek(), Some(ch) if ch.is_ascii_alphanumeric() || ch == '_') {
            self.advance();
        }

        let identifier = &self.source[self.current_span()];
        let kind = KEYWORDS
            .get(identifier)
            .cloned()
            .unwrap_or(TokenKind::Identifier(identifier.to_string()));

        self.add_token(kind, self.current_span());
    }

    fn lex_number(&mut self) {
        while matches!(self.peek(), Some(ch) if ch.is_ascii_digit() || ch == '_') {
            self.advance();
        }

        let kind =
            if self.peek() == Some('.') && self.peek_next().is_some_and(|ch| ch.is_ascii_digit()) {
                self.advance();
                while matches!(self.peek(), Some(ch) if ch.is_ascii_digit() || ch == '_') {
                    self.advance();
                }
                let lexeme = &self.source[self.current_span()].parse::<f64>().unwrap();
                TokenKind::FloatLiteral(*lexeme)
            } else {
                let lexeme = &self.source[self.current_span()].parse::<i64>().unwrap();
                TokenKind::IntLiteral(*lexeme)
            };

        self.add_token(kind, self.current_span());
    }

    fn emit_inferred_semicolon(&mut self) {
        if self
            .tokens
            .last()
            .is_some_and(|tok| tok.kind().infers_semicolon())
        {
            self.add_token(TokenKind::SemiColon, self.current_span());
        }
    }

    fn current_span(&self) -> Span {
        self.start..self.current
    }

    fn match_ch(&mut self, ch: char) -> bool {
        if self.peek().is_some_and(|ch2| ch == ch2) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.source_iter.peek().copied()
    }

    fn peek_next(&mut self) -> Option<char> {
        self.source_iter.clone().nth(1)
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source_iter.next();
        if let Some(ch) = ch {
            self.current += ch.len_utf8();
        }

        ch
    }

    fn current_module(&self) -> &Module {
        self.ctx.module_by_id(self.module_id)
    }

    fn current_module_mut(&mut self) -> &mut Module {
        self.ctx.module_by_id_mut(self.module_id)
    }
}
