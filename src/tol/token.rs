use std::ops::Range;

pub type Span = Range<usize>;

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
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Ang,
    Print,

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

    Indent,
    Dedent,

    Eof,
}

impl TokenKind {
    pub fn infers_semicolon(&self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            RParen | RSquare | Identifier(_) | IntLiteral(_) | FloatLiteral(_)
        )
    }
}
