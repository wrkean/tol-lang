use crate::tol::token::TokenKind;

/// Determines the precedence of the given token kind
pub fn precedence(kind: &TokenKind) -> u8 {
    use TokenKind::*;
    match kind {
        Equal => 1,
        TokenKind::EqualEq | TokenKind::NotEq => 2,
        TokenKind::Greater | TokenKind::GreatEq | TokenKind::Lesser | TokenKind::LessEq => 3,
        Plus | PlusPlus | Minus => 4,
        Star | Slash => 5,
        TokenKind::LParen => 6,
        _ => 0,
    }
}

/// Determines the associativity of the given token kind
pub fn associativity(kind: &TokenKind) -> Associativity {
    use TokenKind::*;
    match kind {
        Plus | Minus | Star | Slash | EqualEq | NotEq | Greater | GreatEq | Lesser | LessEq
        | LParen => Associativity::Left,
        _ => Associativity::Right,
    }
}

/// An operator's associativity
pub enum Associativity {
    Left,
    Right,
}
