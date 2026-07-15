use crate::tol::{token::Span, types::TolType};

pub type SymbolId = usize;

/// A type representing named objects
pub struct Symbol {
    name: String,
    kind: SymbolKind,

    // Span to where it is declared
    span: Span,

    storage: Storage,
}

impl Symbol {
    pub fn new(name: String, span: Span, storage: Storage, kind: SymbolKind) -> Self {
        Self {
            name,
            kind,
            span,
            storage,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &SymbolKind {
        &self.kind
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }
}

pub enum SymbolKind {
    Name { is_mutable: bool, ty: TolType },
}

pub type StorageId = usize;

pub enum Storage {
    Global(StorageId),
    Local(StorageId),
}
