use std::collections::{HashMap, HashSet};

use crate::{
    analyze::UpvalueDesc,
    module::ModuleId,
    parse::ast::stmt::Param,
    prelude::Spanned,
    tol::{token::Span, types::TolType},
};

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

    pub fn set_frame_size(&mut self, frame_size: usize) {
        let SymbolKind::Function {
            frame_size: fsize, ..
        } = &mut self.kind
        else {
            unimplemented!()
        };

        *fsize = frame_size;
    }

    pub fn frame_size(&self) -> usize {
        let SymbolKind::Function { frame_size, .. } = &self.kind else {
            unimplemented!()
        };

        *frame_size
    }

    pub fn kind_mut(&mut self) -> &mut SymbolKind {
        &mut self.kind
    }
}

pub enum SymbolKind {
    Name {
        ty: TolType,
    },
    Function {
        param_types: Spanned<Vec<TolType>>,
        upvalues: Vec<UpvalueDesc>,
        ret_ty: TolType,
        frame_size: usize,
    },
    NativeFunction,
    Klase {
        methods: HashSet<String>,
    },
    Module {
        module_id: ModuleId,
    },
}

pub type StorageId = usize;

pub enum Storage {
    Global(StorageId),
    Local(StorageId),
}
