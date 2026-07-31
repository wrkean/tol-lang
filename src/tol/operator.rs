use crate::tol::token::TokenKind;

/// Determines the precedence of the given token kind
pub fn precedence(kind: &TokenKind) -> u8 {
    match kind {
        TokenKind::Equal
        | TokenKind::PlusEq
        | TokenKind::MinusEq
        | TokenKind::StarEq
        | TokenKind::SlashEq
        | TokenKind::PercentEq => 1,
        TokenKind::AtKw => 2,
        TokenKind::O => 3,
        TokenKind::EqualEq | TokenKind::NotEq => 4,
        TokenKind::Greater | TokenKind::GreatEq | TokenKind::Lesser | TokenKind::LessEq => 5,
        TokenKind::Plus | TokenKind::PlusPlus | TokenKind::Minus => 6,
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => 7,
        TokenKind::LParen | TokenKind::LSquare => 8,
        TokenKind::Dot => 9,
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
