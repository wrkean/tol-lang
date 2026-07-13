use std::sync::Arc;

use crate::tol::token::Span;

pub mod miette_diagnostic;

pub struct TolDiagnostic {
    source: Arc<str>,
    filename: String,
    message: String,
    help: Option<String>,
    severity: Severity,
    labels: Vec<Label>,
}

impl TolDiagnostic {
    pub fn err(source: Arc<str>, filename: String, message: String) -> Self {
        Self {
            source,
            filename,
            message,
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

pub enum Severity {
    Error,
    Warning,
    Advice,
}

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

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());

        self
    }
}
