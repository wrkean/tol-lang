use std::{mem, path::PathBuf, sync::Arc};

use crate::{
    parse::ast::{Ast, stmt::Stmt},
    tol::{
        diagnostic::{Severity, TolDiagnostic, miette_diagnostic::MietteDiagnostic},
        token::Span,
    },
};

pub type ModuleId = usize;

/// Holds all the information of a file.
///
/// In tol, each file is a module
pub struct Module {
    // This module's path
    path: PathBuf,

    // This module's name, derived from the path
    name: String,

    // The atomically referenced counted source
    source: Arc<str>,

    // The compilation state of this module
    compile_state: ModuleCompileState,

    // The Abstract Syntax Tree for this module
    ast: Ast,

    // Diagnostics accumulated during the compilation of this module
    diagnostics: Vec<TolDiagnostic>,

    // True when one of the diagnostic has a severity = Error
    has_an_error: bool,

    // Offsets that points to where each line starts begin, used for determining the line from the
    // given offset in the compiler during runtime
    line_starts: Vec<usize>,
}

impl Module {
    /// Creates a new module derived from the given arguments
    pub fn new(path: PathBuf, name: String, source: String) -> Self {
        // Initialize line_starts
        let mut line_starts = vec![0];

        for (i, ch) in source.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }

        Self {
            path,
            name,
            source: Arc::from(source),
            compile_state: ModuleCompileState::Initialized,
            ast: Ast::new(),
            diagnostics: Vec::new(),
            has_an_error: false,
            line_starts,
        }
    }

    /// Get the source
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get an Arc clone of the source
    pub fn source_arc(&self) -> Arc<str> {
        self.source.clone()
    }

    /// Sets the compile state of the module
    pub fn set_compile_state(&mut self, compile_state: ModuleCompileState) {
        self.compile_state = compile_state;
    }

    /// Adds a new statement to the ast
    pub fn add_statement(&mut self, statement: Stmt) {
        self.ast.push(statement);
    }

    /// Adds a new diagnostic to this module
    pub fn add_diagnostic(&mut self, diagnostic: TolDiagnostic) {
        if diagnostic.severity() == &Severity::Error {
            self.has_an_error = true;
        }

        self.diagnostics.push(diagnostic);
    }

    /// The module name + .tol extension
    pub fn filename(&self) -> String {
        self.name.clone() + ".tol"
    }

    /// This module has an error if one of the pushed diagnostic has a severity = Error
    pub fn has_an_error(&self) -> bool {
        self.has_an_error
    }

    /// Reports the diagnostics for this module one by one
    ///
    /// NOTE: This consumes this module's diagnostics. As such, this is only to be called one
    /// per module as the diagnostics are already consumed at the point when this is accessed again
    pub fn report_diagnostics(&mut self) {
        let diagnostics = mem::take(&mut self.diagnostics);
        for diagnostic in diagnostics {
            eprintln!(
                "{:?}",
                miette::Report::new(MietteDiagnostic::from(diagnostic))
            );
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Takes the ast with std::mem::take
    ///
    /// This should be often times temporary and only used for resolving borrow issues
    pub fn take_ast(&mut self) -> Ast {
        mem::take(&mut self.ast)
    }

    /// Returns the line number in the source code from the given offset
    pub fn line_of(&self, offset: usize) -> usize {
        self.line_starts.partition_point(|&x| x <= offset)
    }

    pub fn line_span(&self, line: usize) -> Span {
        let source = self.source();
        let mut current_line = 1;
        let mut line_start = 0;

        for (i, ch) in source.char_indices() {
            if current_line == line {
                // found the start of the target line, now find where it ends
                let line_end = source[i..]
                    .find('\n')
                    .map(|rel| i + rel)
                    .unwrap_or(source.len());
                return line_start..line_end;
            }
            if ch == '\n' {
                current_line += 1;
                line_start = i + 1;
            }
        }

        line_start..source.len() // fallback: last line, no trailing newline
    }

    pub fn set_ast(&mut self, ast: Ast) {
        self.ast = ast;
    }

    pub fn ast(&self) -> &[Stmt] {
        &self.ast
    }
}

/// A module's compile state, composed of three states:
///
/// - Initialized: Initial state of the module upon creating it
/// - Compiling: State of the module if it is being currently compiled (being lexed/parsed/analyzed/compiled)
/// - Compiled: State of the module after being compiled, which holds its own bytecode
pub enum ModuleCompileState {
    Initialized,
    Compiling,
    Compiled,
}
