use crate::tol::diagnostic::TolDiagnostic;

pub type DiagResult<T> = Result<T, Box<TolDiagnostic>>;
