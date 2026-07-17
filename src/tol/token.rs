use std::{fmt, ops::Range};

pub type Span = Range<usize>;

#[derive(Debug, Clone)]
pub struct Token {
    span: Span,
    kind: TokenKind,
}

impl Token {
    pub fn new(span: Span, kind: TokenKind) -> Self {
        Self { span, kind }
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Ang,
    Print,
    Iiba,
    Paraan,
    Kung,
    Kundi,
    Kungwala,
    Habang,
    Biyakin,
    Ituloy,
    Ibalik,

    SemiColon,
    Colon,
    Equal,
    Plus,
    Minus,
    Star,
    Slash,

    IntLiteral(i64),
    FloatLiteral(f64),
    Identifier(String),

    LParen,
    LSquare,
    RParen,
    RSquare,
    LBrace,
    RBrace,
    NotEq,
    EqualEq,
    GreatEq,
    Greater,
    LessEq,
    Lesser,
    Comma,
    ThinArrow,

    Indent,
    Dedent,

    Eof,
}

impl TokenKind {
    pub fn infers_semicolon(&self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            RParen
                | RSquare
                | Identifier(_)
                | IntLiteral(_)
                | FloatLiteral(_)
                | Biyakin
                | Ituloy
                | Ibalik
        )
    }

    pub fn is_synchronization_point(&self) -> bool {
        use TokenKind::*;
        matches!(self, Ang | Print | Paraan | Biyakin | Ituloy | Ibalik)
    }
}
