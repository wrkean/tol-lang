use crate::tol::token::TokenKind;

/// Determines the precedence of the given token kind
pub fn precedence(kind: &TokenKind) -> u8 {
    use TokenKind::*;
    match kind {
        Equal => 1,
        TokenKind::EqualEq | TokenKind::NotEq => 2,
        TokenKind::Greater | TokenKind::GreatEq | TokenKind::Lesser | TokenKind::LessEq => 3,
        Plus | Minus => 4,
        Star | Slash => 5,
        _ => 0,
    }
}

/// Determines the associativity of the given token kind
pub fn associativity(kind: &TokenKind) -> Associativity {
    use TokenKind::*;
    match kind {
        Plus
        | Minus
        | Star
        | Slash
        | TokenKind::EqualEq
        | TokenKind::NotEq
        | TokenKind::Greater
        | TokenKind::GreatEq
        | TokenKind::Lesser
        | TokenKind::LessEq => Associativity::Left,
        _ => Associativity::Right,
    }
}

/// An operator's associativity
pub enum Associativity {
    Left,
    Right,
}
