use std::sync::Arc;

use crate::{
    tol::{diagnostic::Label, token::Span},
    vm::value::ValueError,
};

/// Diagnostic struct used to construct diagnostics at runtime
#[derive(Debug)]
pub struct RuntimeError {
    pub source: Arc<str>,
    pub filename: String,
    pub message: String,
    pub help: Option<String>,
    pub label: Label,
}

impl RuntimeError {
    pub fn new(
        source: Arc<str>,
        filename: impl Into<String>,
        message: impl Into<String>,
        label: Label,
    ) -> Self {
        Self {
            source,
            filename: filename.into(),
            message: message.into(),
            label,
            help: None,
        }
    }

    pub fn help(mut self, message: impl Into<String>) -> Self {
        self.help = Some(message.into());

        self
    }

    pub fn from_value_error(
        value_error: ValueError,
        source: Arc<str>,
        filename: impl Into<String>,
        label: Label,
    ) -> Self {
        Self {
            source,
            filename: filename.into(),
            message: value_error.message,
            help: value_error.help,
            label,
        }
    }
}
