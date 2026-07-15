use phf::phf_map;

use crate::tol::token::TokenKind;

pub static KEYWORDS: phf::Map<&'static str, TokenKind> = phf_map! {
    "ang" => TokenKind::Ang,
    "print" => TokenKind::Print,
    "iiba" => TokenKind::Iiba,
};
