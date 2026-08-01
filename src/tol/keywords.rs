use phf::phf_map;

use crate::tol::token::TokenKind;

pub static KEYWORDS: phf::Map<&'static str, TokenKind> = phf_map! {
    "ang" => TokenKind::Ang,
    "paraan" => TokenKind::Paraan,
    "kung" => TokenKind::Kung,
    "kundi" => TokenKind::Kundi,
    "kungwala" => TokenKind::Kungwala,
    "habang" => TokenKind::Habang,
    "biyakin" => TokenKind::Biyakin,
    "ituloy" => TokenKind::Ituloy,
    "ibalik" => TokenKind::Ibalik,
    "klase" => TokenKind::Klase,
    "kunin" => TokenKind::Kunin,
    "totoo" => TokenKind::Totoo,
    "mali" =>  TokenKind::Mali,
    "at" => TokenKind::AtKw,
    "o" => TokenKind::O,
    "bawat" => TokenKind::Bawat,
    "sa" => TokenKind::Sa,
    "di" => TokenKind::Di,
};
