use std::sync::Arc;

use crate::tol::token::Span;

pub mod miette_diagnostic;

#[derive(Debug)]
pub struct TolDiagnostic {
    source: Arc<str>,
    filename: String,
    message: String,
    help: Option<String>,
    severity: Severity,
    labels: Vec<Label>,
}

impl TolDiagnostic {
    pub fn err(source: Arc<str>, filename: String, message: impl Into<String>) -> Self {
        Self {
            source,
            filename,
            message: message.into(),
            help: None,
            severity: Severity::Error,
            labels: Vec::new(),
        }
    }

    pub fn label(mut self, label: Label) -> Self {
        self.labels.push(label);

        self
    }

    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());

        self
    }
}

#[derive(Debug)]
pub enum Severity {
    Error,
    Warning,
    Advice,
}

#[derive(Debug)]
pub struct Label {
    span: Span,
    message: Option<String>,
}

impl Label {
    pub fn new(span: Span) -> Self {
        Self {
            span,
            message: None,
        }
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());

        self
    }
}

pub mod predefined_diagnostics {
    use crate::{
        global_ctx::Module,
        tol::{
            diagnostic::{Label, TolDiagnostic},
            token::Span,
        },
    };

    pub fn unexpected_token(
        current_module: &Module,
        message: impl Into<String>,
        label_span: Span,
    ) -> TolDiagnostic {
        TolDiagnostic::err(
            current_module.source_arc(),
            current_module.filename(),
            "may nakita akong hindi inaasahang token",
        )
        .label(Label::new(label_span).message(message.into()))
    }
}
