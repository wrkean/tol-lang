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

    /// Get the lexeme of identifiers ONLY
    pub fn lexeme(&self) -> &str {
        let TokenKind::Identifier(lexeme) = self.kind() else {
            unimplemented!()
        };

        lexeme
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Ang,
    Paraan,
    Kung,
    Kundi,
    Kungwala,
    Habang,
    Biyakin,
    Ituloy,
    Ibalik,
    Klase,
    Kunin,

    SemiColon,
    Colon,
    Equal,
    Plus,
    PlusEq,
    PlusPlus,
    Minus,
    MinusEq,
    Star,
    StarEq,
    Slash,
    SlashEq,
    Percent,
    PercentEq,
    Dot,

    IntLiteral(i64),
    FloatLiteral(f64),
    Identifier(String),
    StringLiteral(String),

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
    ThickArrow,
    Pipe,
    At,

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
                | RBrace
                | Identifier(_)
                | IntLiteral(_)
                | FloatLiteral(_)
                | StringLiteral(_)
                | Biyakin
                | Ituloy
                | Ibalik
        )
    }

    pub fn is_synchronization_point(&self) -> bool {
        use TokenKind::*;
        matches!(self, Ang | Paraan | Biyakin | Ituloy | Ibalik)
    }
}
