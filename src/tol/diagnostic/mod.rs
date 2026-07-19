use std::sync::Arc;

use crate::tol::token::Span;

pub mod miette_diagnostic;
pub mod runtime;
/// Diagnostic struct used to construct diagnostics at compile time
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

    /// Converts a byte offset into (line, column), both 1-indexed.
    fn line_col(&self, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;

        for (i, ch) in self.source.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    /// Extracts the full line of source text containing the given byte offset.
    fn line_text(&self, offset: usize) -> &str {
        let start = self.source[..offset]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let end = self.source[offset..]
            .find('\n')
            .map(|i| offset + i)
            .unwrap_or(self.source.len());
        &self.source[start..end]
    }

    /// Formats this diagnostic into a human-readable string:
    /// <severity>
    /// [<filename>:<line>:<column>]
    /// <source line for each label>
    /// <label message, if any>
    ///
    /// <help, if any>
    pub fn simple_report(&self) -> String {
        let mut out = String::new();

        // Header: severity
        out.push_str(&format!("{}: {}", self.severity.as_str(), &self.message));
        out.push('\n');

        // Location: use the first label's span for the primary position,
        // falling back to (1, 1) if there are no labels.
        let (line, col) = self
            .labels
            .first()
            .map(|l| self.line_col(l.span.start))
            .unwrap_or((1, 1));

        out.push_str(&format!("[{}:{}:{}]\n\n", self.filename, line, col));

        // Labeled source lines
        for label in &self.labels {
            print!("|    ");
            let text = self.line_text(label.span.start);
            out.push_str(text);
            out.push('\n');

            if let Some(msg) = &label.message {
                out.push_str(&format!("{} {}", "^".repeat(text.len()), msg));
                out.push('\n');
            }
        }

        // Help section
        if let Some(help) = &self.help {
            out.push('\n');
            out.push_str(&format!("tulong: {}", help));
            out.push('\n');
        }

        out
    }
}

/// The severity of the diagnostic
#[derive(Debug, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Advice,
}

impl Severity {
    fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Advice => "advice",
        }
    }
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
