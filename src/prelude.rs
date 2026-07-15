use crate::tol::{diagnostic::TolDiagnostic, token::Span};

pub type DiagResult<T> = Result<T, Box<TolDiagnostic>>;

pub struct Spanned<T> {
    span: Span,
    item: T,
}

impl<T> Spanned<T> {
    pub fn new(span: Span, item: T) -> Self {
        Self { span, item }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn item(&self) -> &T {
        &self.item
    }
}
