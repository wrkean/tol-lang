use crate::tol::token::TokenKind;

/// Determines the precedence of the given token kind
pub fn precedence(kind: &TokenKind) -> u8 {
    match kind {
        TokenKind::Equal => 1,
        TokenKind::EqualEq | TokenKind::NotEq => 2,
        TokenKind::Greater | TokenKind::GreatEq | TokenKind::Lesser | TokenKind::LessEq => 3,
        TokenKind::Plus | TokenKind::PlusPlus | TokenKind::Minus => 4,
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => 5,
        TokenKind::LParen | TokenKind::LSquare => 6,
        TokenKind::Dot => 7,
        _ => 0,
    }
}

/// Determines the associativity of the given token kind
pub fn associativity(kind: &TokenKind) -> Associativity {
    use TokenKind::*;
    match kind {
        TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::Slash
        | TokenKind::EqualEq
        | TokenKind::NotEq
        | TokenKind::Greater
        | TokenKind::GreatEq
        | TokenKind::Lesser
        | TokenKind::LessEq
        | TokenKind::LParen
        | TokenKind::Dot
        | TokenKind::LSquare => Associativity::Left,
        _ => Associativity::Right,
    }
}

/// An operator's associativity
pub enum Associativity {
    Left,
    Right,
}
