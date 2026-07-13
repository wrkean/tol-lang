//! Module for implementing a trait to turn TolDiagnostic into a MietteDiagnostic for beautiful
//! error reporting

use std::sync::Arc;

use miette::{LabeledSpan, NamedSource};

use crate::tol::diagnostic::TolDiagnostic;

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
#[error("{message}")]
pub struct MietteDiagnostic {
    message: String,

    #[source_code]
    src: NamedSource<Arc<str>>,

    #[label(collection)]
    labels: Vec<LabeledSpan>,

    #[help]
    help: Option<String>,
}

impl From<TolDiagnostic> for MietteDiagnostic {
    fn from(tol_diag: TolDiagnostic) -> Self {
        let labels = tol_diag
            .labels
            .into_iter()
            .map(|label| {
                LabeledSpan::new(
                    label.message,
                    label.span.start,
                    label.span.end - label.span.start,
                )
            })
            .collect();

        Self {
            message: match tol_diag.severity {
                super::Severity::Error => format!("error: {}", tol_diag.message),
                super::Severity::Warning => todo!("babala: {}", tol_diag.message),
                super::Severity::Advice => todo!("abiso: {}", tol_diag.message),
            },
            src: NamedSource::new(tol_diag.filename.clone(), tol_diag.source),
            labels,
            help: tol_diag.help,
        }
    }
}
