use crate::tol::token::TokenKind;

/// Determines the precedence of the given token kind
pub fn precedence(kind: &TokenKind) -> u8 {
    use TokenKind::*;
    match kind {
        Plus | Minus => 1,
        Star | Slash => 2,
        _ => 0,
    }
}

/// Determines the associativity of the given token kind
pub fn associativity(kind: &TokenKind) -> Associativity {
    use TokenKind::*;
    match kind {
        Plus | Minus | Star | Slash => Associativity::Left,
        _ => Associativity::Right,
    }
}

/// An operator's associativity
pub enum Associativity {
    Left,
    Right,
}
