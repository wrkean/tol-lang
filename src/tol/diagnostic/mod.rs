use std::sync::Arc;

use crate::tol::token::Span;

pub mod miette_diagnostic;

/// Diagnostic struct used to construct diagnostics at error site
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
    /// Generate a new diagnostic with severity = Error
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

    /// Attach a label to this diagnostic
    pub fn label(mut self, label: Label) -> Self {
        self.labels.push(label);

        self
    }

    /// Attach a help message to this diagnostic
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());

        self
    }

    pub fn severity(&self) -> &Severity {
        &self.severity
    }
}

/// The severity of the diagnostic
#[derive(Debug, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Advice,
}

/// Label pointing to the diagnostic, does not own the source. We are responsible for pointing this
/// label to the correct source
#[derive(Debug)]
pub struct Label {
    span: Span,
    message: Option<String>,
}

impl Label {
    /// Generate a new label
    pub fn new(span: Span) -> Self {
        Self {
            span,
            message: None,
        }
    }

    /// Attach a message to this label
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());

        self
    }
}

pub mod predefined_diagnostics {
    //! Module containing functions that construct predefined TolDiagnostic like unexpected token errors

    use crate::{
        module::Module,
        tol::{
            diagnostic::{Label, TolDiagnostic},
            token::Span,
            types,
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

    pub fn unrecognized_type(current_module: &Module, label_span: Span) -> TolDiagnostic {
        TolDiagnostic::err(
            current_module.source_arc(),
            current_module.filename(),
            "may nakita akong invalid na tipo",
        )
        .label(Label::new(label_span).message("hindi makilala ang tipo na ito"))
        .help(format!(
            "mga halimbawa ng tipo sa tol: {}",
            types::type_list().join(",")
        ))
    }

    pub fn use_of_undeclared_variable(current_module: &Module, label_span: Span) -> TolDiagnostic {
        TolDiagnostic::err(
            current_module.source_arc(),
            current_module.filename(),
            "paggamit ng hindi pa na-ideklarang variable",
        )
        .label(Label::new(label_span).message("hindi mo pa ito na-ideklara"))
        .help("kailangan munang ma-ideklara ang variable bago mo ito magamit")
    }

    pub fn unclosed_string_literal(current_module: &Module, label_span: Span) -> TolDiagnostic {
        TolDiagnostic::err(
            current_module.source_arc(),
            current_module.filename(),
            "hindi na-isaradong string",
        )
        .label(Label::new(label_span).message("hindi ito na-isarado"))
    }
}
