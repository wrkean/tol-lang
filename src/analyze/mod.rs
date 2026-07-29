//! Module responsible for Abstract Syntax Tree analysis

use std::collections::{HashMap, HashSet};

use crate::{
    analyze::symbol::{Storage, Symbol, SymbolId, SymbolKind},
    global_ctx::GlobalContext,
    module::{Module, ModuleId},
    parse::ast::{
        Ast,
        expr::{Expr, ExprKind},
        stmt::{Stmt, StmtKind},
    },
    prelude::DiagResult,
    tol::{
        diagnostic::{Label, TolDiagnostic, predefined_diagnostics},
        token::{Token, TokenKind},
        types::TolType,
    },
};

pub mod symbol;

/// Represents the resolved variable upon declaration of a variable, only those with names can have
/// this. This enum is used to index into the the symbol table in which the symbol itself can index
/// into the module globals or function locals, or the vm upvalues if it is resolved as an upvalue
#[derive(Debug, Clone)]
pub enum ResolvedVar {
    /// Local variable, with index pointing to its symbol
    Local(SymbolId),

    /// Global variable, with index pointing to its symbol
    Global(SymbolId),

    /// An upvalue, with index pointing to the vm upvalues
    Upvalue(usize),
}

impl ResolvedVar {
    /// If it is a local or global, return the symbol it is pointing to. The upvalue variant is
    /// unimplemented for ease of use (like having to unwrap or handle the Option enum everytime we
    /// want the symbol id which — all the time, I know if its either a local or a global and not an
    /// upvalue).
    pub fn symbol_id(&self) -> SymbolId {
        let (ResolvedVar::Local(id) | ResolvedVar::Global(id)) = self else {
            unimplemented!()
        };

        *id
    }
}

/// Contains the context for the function definition
pub struct FunctionCtx {
    /// Scopes enclosed by this function
    scopes: Vec<Scope>,

    /// List of upvalue descriptions
    upvalues: Vec<UpvalueDesc>,
}

impl FunctionCtx {
    /// Creates a new function context
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            upvalues: Vec::new(),
        }
    }
}

/// Description of an upvalue, to be used later in the compiler
pub struct UpvalueDesc {
    /// True if this upvalue is local to the immediate enclosing function, otherwise false.
    pub is_local: bool,

    /// Index pointing to the list of upvalues in the vm
    pub index: usize,
}

#[derive(Debug)]
struct Scope {
    symbols: HashMap<String, SymbolId>,
    is_function_scope: bool,
    slot_start: usize,
}

impl Scope {
    fn new(is_function_scope: bool, slot_start: usize) -> Self {
        Self {
            symbols: HashMap::new(),
            is_function_scope,
            slot_start,
        }
    }

    fn get(&self, symbol_name: &str) -> Option<&SymbolId> {
        self.symbols.get(symbol_name)
    }

    fn insert(&mut self, symbol_name: String, id: SymbolId) {
        self.symbols.insert(symbol_name, id);
    }
}

/// Analyzes the target module's ast
pub struct Analyzer<'gctx> {
    /// Function context stack
    functions: Vec<FunctionCtx>,

    /// The global context
    ctx: &'gctx mut GlobalContext,

    /// ID pointing to the current module being analyzed in the global context
    module_id: ModuleId,

    /// How deep is the analyzer inside a loop. Used for determining whether a continue or break
    /// statement is valid at compile time
    loop_depth: u8,

    /// The next global slot
    next_global_slot: usize,

    /// The next local slot
    next_local_slot: usize,

    /// The maximum amount of locals the current function has had. Used to determine the frame size
    /// of a function at compile time
    max_local_slot: usize,

    /// The next local slot stack
    next_local_slot_stack: Vec<usize>,

    /// The max local slot stack
    max_local_slot_stack: Vec<usize>,
}

impl<'gctx> Analyzer<'gctx> {
    /// Creates a new analyze instance that targets the given module by id
    pub fn new(ctx: &'gctx mut GlobalContext, module_id: ModuleId) -> Self {
        Self {
            functions: vec![FunctionCtx::new()],
            ctx,
            module_id,
            loop_depth: 0,
            next_global_slot: 0,
            next_local_slot: 1,
            max_local_slot: 1,
            next_local_slot_stack: Vec::new(),
            max_local_slot_stack: Vec::new(),
        }
    }

    /// Runs the analyzer on the target module
    pub fn analyze(&mut self) {
        self.enter_scope(false);

        self.define_native("input");
        self.define_native("alis");
        self.define_native("print");
        self.define_native("println");

        self.resolve_names();

        self.exit_scope();
    }

    fn define_native(&mut self, name: impl Into<String>) -> DiagResult<()> {
        let storage = self.assign_storage();
        let Storage::Global(id) = storage else {
            unreachable!()
        };
        let name = name.into();
        let symbol = Symbol::new(name.clone(), 0..0, storage, SymbolKind::NativeFunction);
        self.declare_symbol(symbol)?;
        self.ctx.new_native_fn(name, id);

        Ok(())
    }

    fn resolve_names(&mut self) {
        let mut ast = self.current_module_mut().take_ast();
        for statement in ast.iter_mut() {
            if let Err(diag) = self.resolve_statement(statement) {
                self.current_module_mut().add_diagnostic(*diag);
            }
        }
        self.current_module_mut().set_ast(ast);
    }

    fn resolve_statement(&mut self, statement: &mut Stmt) -> DiagResult<()> {
        match statement.kind_mut() {
            StmtKind::Ang { .. } => self.resolve_ang(statement),
            StmtKind::Paraan { .. } => self.resolve_paraan(statement),
            StmtKind::Expr { expr } => self.resolve_expression(expr),
            StmtKind::Kung { .. } => self.resolve_kung(statement),
            StmtKind::Habang { .. } => self.resolve_habang(statement),
            StmtKind::Biyakin => self.resolve_biyakin(statement),
            StmtKind::Ituloy => self.resolve_ituloy(statement),
            StmtKind::Ibalik { .. } => self.resolve_ibalik(statement),
            StmtKind::Klase { .. } => self.resolve_klase(statement),
            StmtKind::Block { statements } => {
                self.enter_scope(false);
                for statement in statements {
                    if let Err(diag) = self.resolve_statement(statement) {
                        self.current_module_mut().add_diagnostic(*diag);
                    }
                }

                self.exit_scope();

                Ok(())
            }
        }
    }

    fn resolve_ang(&mut self, ang: &mut Stmt) -> DiagResult<()> {
        let StmtKind::Ang { name, ty, rhs } = ang.kind_mut() else {
            unreachable!()
        };

        self.resolve_expression(rhs)?;
        let storage = self.assign_storage();

        // Should be an identifier if the parser is a good boy
        let TokenKind::Identifier(symbol_name) = name.kind() else {
            unreachable!()
        };
        let symbol = Symbol::new(
            symbol_name.clone(),
            name.span().clone(),
            storage,
            SymbolKind::Name { ty: ty.clone() },
        );

        let var = self.declare_symbol(symbol)?;

        // This ast node is now pointing it's symbol id to its declaration in the symbol table
        ang.set_resolved_var(var);

        Ok(())
    }

    fn resolve_paraan(&mut self, paraan: &mut Stmt) -> DiagResult<()> {
        let StmtKind::Paraan {
            name,
            params,
            ret_ty,
            block,
        } = paraan.kind_mut()
        else {
            unreachable!()
        };

        let storage = self.assign_storage();
        let TokenKind::Identifier(symbol_name) = name.kind() else {
            unreachable!()
        };
        let symbol = Symbol::new(
            symbol_name.clone(),
            name.span().clone(),
            storage,
            SymbolKind::Function {
                param_types: params.spanned_types(),
                ret_ty: ret_ty.clone(),
                frame_size: 0,
                upvalues: Vec::new(),
            },
        );
        let resolved_var = self.declare_symbol(symbol)?;

        self.enter_function();
        self.enter_scope(true);

        for param in params.params.iter() {
            let TokenKind::Identifier(param_name) = param.name.kind() else {
                unreachable!()
            };
            let symbol = Symbol::new(
                param_name.clone(),
                param.span.clone(),
                self.assign_storage(),
                SymbolKind::Name {
                    ty: param.ty.clone(),
                },
            );

            if let Err(diag) = self.declare_symbol(symbol) {
                self.current_module_mut().add_diagnostic(*diag);
            }
        }

        self.resolve_statement(block)?;

        self.exit_scope();
        let (frame_size, upvalues) = self.exit_function();

        let id = resolved_var.symbol_id();
        self.ctx.symbol_by_id_mut(id).set_frame_size(frame_size);
        let SymbolKind::Function { upvalues: uvs, .. } = self.ctx.symbol_by_id_mut(id).kind_mut()
        else {
            unreachable!()
        };
        *uvs = upvalues;

        paraan.set_resolved_var(resolved_var);

        Ok(())
    }

    fn resolve_kung(&mut self, kung: &mut Stmt) -> DiagResult<()> {
        let StmtKind::Kung {
            then_branches,
            else_branch,
        } = kung.kind_mut()
        else {
            unreachable!()
        };

        for then in then_branches {
            self.resolve_expression(then.condition.as_mut().unwrap())?;
            self.resolve_statement(&mut then.block)?;
        }

        if let Some(else_) = else_branch {
            self.resolve_statement(&mut else_.block)?;
        }

        Ok(())
    }

    fn resolve_habang(&mut self, habang: &mut Stmt) -> DiagResult<()> {
        let StmtKind::Habang { condition, block } = habang.kind_mut() else {
            unreachable!()
        };

        self.resolve_expression(condition)?;
        self.loop_depth += 1;
        self.resolve_statement(block)?;
        self.loop_depth -= 1;

        Ok(())
    }

    fn resolve_biyakin(&mut self, biyakin: &Stmt) -> DiagResult<()> {
        if self.loop_depth == 0 {
            let current_module = self.current_module();
            let diagnostic = TolDiagnostic::err(
                current_module.source_arc(),
                current_module.filename(),
                "paggamit ng `biyakin` sa labas ng loop",
            )
            .label(Label::new(biyakin.span().clone()).message("ito ay nasa labas ng loop"))
            .help("maaari lamang gamitin ang `biyakin` sa loob ng loop");

            return Err(Box::new(diagnostic));
        }

        Ok(())
    }

    fn resolve_ituloy(&mut self, ituloy: &Stmt) -> DiagResult<()> {
        if self.loop_depth == 0 {
            let current_module = self.current_module();
            let diagnostic = TolDiagnostic::err(
                current_module.source_arc(),
                current_module.filename(),
                "paggamit ng `ituloy` sa labas ng loop",
            )
            .label(Label::new(ituloy.span().clone()).message("ito ay nasa labas ng loop"))
            .help("maaari lamang gamitin ang `biyakin` sa loob ng loop");

            return Err(Box::new(diagnostic));
        }

        Ok(())
    }

    fn resolve_ibalik(&mut self, ibalik: &mut Stmt) -> DiagResult<()> {
        let StmtKind::Ibalik { expr } = ibalik.kind_mut() else {
            unreachable!()
        };

        if let Some(ex) = expr {
            self.resolve_expression(ex)?;
        }

        if self.next_local_slot_stack.is_empty() {
            let current_module = self.current_module();
            let diagnostic = TolDiagnostic::err(
                current_module.source_arc(),
                current_module.filename(),
                "paggamit ng `ibalik` sa labas ng paraan",
            )
            .label(Label::new(ibalik.span().clone()).message("ito ay nasa labas ng paraan"))
            .help("maaari lamang gamitin ang `ibalik` sa loob ng isang paraan");

            return Err(Box::new(diagnostic));
        }

        Ok(())
    }

    fn resolve_klase(&mut self, klase: &mut Stmt) -> DiagResult<()> {
        let StmtKind::Klase { name, methods } = klase.kind_mut() else {
            unreachable!()
        };

        let storage = self.assign_storage();
        let klase_name = name.lexeme();
        let methods_set = methods
            .iter()
            .map(|s| {
                let StmtKind::Paraan { name, .. } = s.kind() else {
                    unreachable!()
                };

                name.lexeme().to_string()
            })
            .collect::<HashSet<_>>();
        let symbol = Symbol::new(
            klase_name.to_string(),
            name.span().clone(),
            storage,
            SymbolKind::Klase {
                methods: methods_set,
            },
        );
        let resolved_var = self.declare_symbol(symbol)?;

        for method in methods.iter_mut() {
            if let Err(diag) = self.resolve_method(method) {
                self.current_module_mut().add_diagnostic(*diag);
            }
        }

        klase.set_resolved_var(resolved_var);
        Ok(())
    }

    fn resolve_method(&mut self, method: &mut Stmt) -> DiagResult<()> {
        let StmtKind::Paraan {
            name,
            params,
            ret_ty,
            block,
        } = method.kind_mut()
        else {
            unreachable!()
        };

        let storage = self.assign_storage();
        let TokenKind::Identifier(symbol_name) = name.kind() else {
            unreachable!()
        };
        let symbol = Symbol::new(
            symbol_name.clone(),
            name.span().clone(),
            storage,
            SymbolKind::Function {
                param_types: params.spanned_types(),
                ret_ty: ret_ty.clone(),
                frame_size: 0,
                upvalues: Vec::new(),
            },
        );

        // 1. Add symbol to GlobalContext WITHOUT inserting into current_scope map
        let id = self.ctx.add_symbol(symbol);
        let resolved_var = if self.is_in_global_scope() {
            ResolvedVar::Global(id)
        } else {
            ResolvedVar::Local(id)
        };

        // 2. Resolve parameters & body inside the method function context
        self.enter_function();
        self.enter_scope(true);

        for param in params.params.iter() {
            let TokenKind::Identifier(param_name) = param.name.kind() else {
                unreachable!()
            };
            let symbol = Symbol::new(
                param_name.clone(),
                param.span.clone(),
                self.assign_storage(),
                SymbolKind::Name {
                    ty: param.ty.clone(),
                },
            );

            if let Err(diag) = self.declare_symbol(symbol) {
                self.current_module_mut().add_diagnostic(*diag);
            }
        }

        self.resolve_statement(block)?;

        self.exit_scope();
        let (frame_size, upvalues) = self.exit_function();

        let symbol = self.ctx.symbol_by_id_mut(id);
        symbol.set_frame_size(frame_size);
        let SymbolKind::Function { upvalues: uvs, .. } = symbol.kind_mut() else {
            unreachable!()
        };
        *uvs = upvalues;

        method.set_resolved_var(resolved_var);

        Ok(())
    }

    fn resolve_expression(&mut self, expression: &mut Expr) -> DiagResult<()> {
        let span = expression.span().clone();
        match expression.kind_mut() {
            ExprKind::Integer(_) => Ok(()),
            ExprKind::Float(_) => Ok(()),
            ExprKind::Str { text, interned_id } => {
                let id = self.ctx.intern(text);
                *interned_id = Some(id);

                Ok(())
            }
            ExprKind::Identifier(ident) => match self.resolve_identifier(ident) {
                Some(var) => {
                    expression.set_resolved_var(var);
                    Ok(())
                }
                None => {
                    let current_module = self.current_module();
                    let diagnostic = predefined_diagnostics::use_of_undeclared_variable(
                        current_module,
                        expression.span().clone(),
                    );

                    Err(Box::new(diagnostic))
                }
            },
            ExprKind::Binary { left, right, op } => {
                if let Err(diag) = self.resolve_expression(left) {
                    self.current_module_mut().add_diagnostic(*diag);
                }

                if matches!(
                    op.kind(),
                    TokenKind::Equal
                        | TokenKind::PlusEq
                        | TokenKind::MinusEq
                        | TokenKind::StarEq
                        | TokenKind::SlashEq
                        | TokenKind::PercentEq
                ) {
                    self.ensure_valid_assignment(left, op)?;
                }

                self.resolve_expression(right)
            }
            ExprKind::Call { left, args } => {
                if !left.is_lvalue() {
                    let current_module = self.current_module();
                    let diagnostic = TolDiagnostic::err(
                        current_module.source_arc(),
                        current_module.filename(),
                        "pag-tawag ng hindi isang \"lvalue\"",
                    )
                    .label(
                        Label::new(left.span().clone())
                            .message("hindi ito isang \"lvalue\", ngunit tinawag mo ito"),
                    )
                    .help("mga \"lvalue\" lamang ang pwede tawagin");

                    return Err(Box::new(diagnostic));
                }

                self.resolve_expression(left)?;

                for arg in args {
                    if let Err(diag) = self.resolve_expression(arg) {
                        self.current_module_mut().add_diagnostic(*diag);
                    }
                }

                Ok(())
            }
            ExprKind::AnonymousFn { params, body } => {
                let storage = self.assign_storage();
                let param_types = params.spanned_types();
                let name = format!("__anonymous_fn_{}_{}__", span.start, span.end,);
                let symbol = Symbol::new(
                    name,
                    span,
                    storage,
                    SymbolKind::Function {
                        param_types,
                        ret_ty: TolType::DiAlam,
                        frame_size: 0,
                        upvalues: Vec::new(),
                    },
                );
                let resolved_var = self.declare_symbol(symbol)?;

                self.enter_function();
                self.enter_scope(true);

                for param in params.params.iter() {
                    let TokenKind::Identifier(param_name) = param.name.kind() else {
                        unreachable!()
                    };
                    let symbol = Symbol::new(
                        param_name.clone(),
                        param.span.clone(),
                        self.assign_storage(),
                        SymbolKind::Name {
                            ty: param.ty.clone(),
                        },
                    );

                    if let Err(diag) = self.declare_symbol(symbol) {
                        self.current_module_mut().add_diagnostic(*diag);
                    }
                }

                self.resolve_expression(body)?;

                self.exit_scope();
                let (frame_size, upvalues) = self.exit_function();

                self.ctx
                    .symbol_by_id_mut(resolved_var.symbol_id())
                    .set_frame_size(frame_size);
                let SymbolKind::Function { upvalues: uvs, .. } = self
                    .ctx
                    .symbol_by_id_mut(resolved_var.symbol_id())
                    .kind_mut()
                else {
                    unreachable!()
                };
                *uvs = upvalues;

                expression.set_resolved_var(resolved_var);

                Ok(())
            }
            ExprKind::FieldAccess { object, field } => {
                self.resolve_expression(object)?;

                Ok(())
            }
            ExprKind::List { elements, .. } => {
                for element in elements.iter_mut() {
                    if let Err(diag) = self.resolve_expression(element) {
                        self.current_module_mut().add_diagnostic(*diag);
                    }
                }

                Ok(())
            }
            ExprKind::IndexAccess { left, index } => {
                self.resolve_expression(left)?;
                self.resolve_expression(index)?;

                Ok(())
            }
        }
    }

    fn ensure_valid_assignment(&mut self, left: &Expr, op: &Token) -> DiagResult<()> {
        let current_module = self.current_module();
        if !left.is_lvalue() {
            let diagnostic = TolDiagnostic::err(
                current_module.source_arc(),
                current_module.filename(),
                "pag-assign sa hindi \"lvalue\"",
            )
            .label(Label::new(left.span().clone()).message("hindi ito isang \"lvalue\""));

            return Err(Box::new(diagnostic));
        }

        Ok(())
    }

    fn declare_symbol(&mut self, symbol: Symbol) -> DiagResult<ResolvedVar> {
        let current_scope = self
            .functions
            .last_mut()
            .unwrap()
            .scopes
            .last_mut()
            .unwrap();
        match current_scope.get(symbol.name()) {
            Some(&id) => {
                let declared_symbol = self.ctx.symbol_by_id(id);
                let declared_span = declared_symbol.span().clone();

                let current_module = self.current_module();
                let diagnostic = TolDiagnostic::err(
                    current_module.source_arc(),
                    current_module.filename(),
                    "pag-deklara ng kaparehong pangalan sa iisang sakop",
                )
                .label(Label::new(declared_span).message("na-ideklara na dito"))
                .label(Label::new(symbol.span().clone()).message("dineklara mo ulit dito"));

                Err(Box::new(diagnostic))
            }
            None => {
                let name = symbol.name().to_string();
                let id = self.ctx.add_symbol(symbol);
                current_scope.insert(name, id);

                if self.is_in_global_scope() {
                    Ok(ResolvedVar::Global(id))
                } else {
                    Ok(ResolvedVar::Local(id))
                }
            }
        }
    }

    fn resolve_identifier(&mut self, name: &str) -> Option<ResolvedVar> {
        self.resolve_in_function(self.functions.len() - 1, name)
    }

    fn resolve_in_function(&mut self, func_idx: usize, name: &str) -> Option<ResolvedVar> {
        // 1. Check local scopes of the function at func_idx
        for scope in self.functions[func_idx].scopes.iter().rev() {
            if let Some(&id) = scope.get(name) {
                if func_idx == 0 {
                    return Some(ResolvedVar::Global(id));
                } else {
                    return Some(ResolvedVar::Local(id));
                }
            }
        }

        // 2. Base case: If we're at the top-level (func_idx == 0) and didn't find it, return None
        if func_idx == 0 {
            return None;
        }

        // 3. Recursively resolve in enclosing outer functions (func_idx - 1)
        match self.resolve_in_function(func_idx - 1, name) {
            Some(ResolvedVar::Local(symbol_id)) => {
                // Extract the local stack slot index from Symbol.storage (not the SymbolId)
                let symbol = self.ctx.symbol_by_id(symbol_id);
                let Storage::Local(slot) = symbol.storage() else {
                    unreachable!()
                };

                let idx = self.add_upvalue(func_idx, *slot, true);
                Some(ResolvedVar::Upvalue(idx))
            }
            Some(ResolvedVar::Upvalue(parent_upvalue_idx)) => {
                let idx = self.add_upvalue(func_idx, parent_upvalue_idx, false);
                Some(ResolvedVar::Upvalue(idx))
            }
            Some(ResolvedVar::Global(symbol_id)) => {
                // Globals do not need upvalues; inner functions access globals directly!
                Some(ResolvedVar::Global(symbol_id))
            }
            None => None,
        }
    }

    fn add_upvalue(&mut self, func_idx: usize, index: usize, is_local: bool) -> usize {
        let upvalues = &mut self.functions[func_idx].upvalues;

        if let Some(index) = upvalues
            .iter()
            .position(|uv| uv.is_local == is_local && uv.index == index)
        {
            return index;
        }

        upvalues.push(UpvalueDesc { is_local, index });
        upvalues.len() - 1
    }

    fn current_module(&self) -> &Module {
        self.ctx.module_by_id(self.module_id)
    }

    fn current_module_mut(&mut self) -> &mut Module {
        self.ctx.module_by_id_mut(self.module_id)
    }

    fn enter_function(&mut self) {
        self.functions.push(FunctionCtx::new());
        self.next_local_slot_stack.push(self.next_local_slot);
        self.max_local_slot_stack.push(self.max_local_slot);
        self.next_local_slot = 1;
        self.max_local_slot = 1;
    }

    fn exit_function(&mut self) -> (usize, Vec<UpvalueDesc>) {
        let upvalues = self.functions.pop().unwrap().upvalues;
        let frame_size = self.max_local_slot;

        self.next_local_slot = self
            .next_local_slot_stack
            .pop()
            .expect("function scope stack underflow");
        self.max_local_slot = self
            .max_local_slot_stack
            .pop()
            .expect("function scope stack underflow");

        (frame_size, upvalues)
    }

    fn enter_scope(&mut self, is_function_scope: bool) {
        let current_fn = self.functions.last_mut().unwrap();
        current_fn
            .scopes
            .push(Scope::new(is_function_scope, self.next_local_slot));
    }

    fn exit_scope(&mut self) {
        let current_fn = self.functions.last_mut().unwrap();
        let scope = current_fn
            .scopes
            .pop()
            .expect("exit_scope called with no active scope");
        if self.functions.len() > 1 {
            self.next_local_slot = scope.slot_start;
        }
    }

    fn is_in_global_scope(&self) -> bool {
        self.functions.len() == 1 && self.functions.first().unwrap().scopes.len() == 1
    }

    fn get_global_slot(&mut self) -> usize {
        let slot = self.next_global_slot;
        self.next_global_slot += 1;

        slot
    }

    fn get_local_slot(&mut self) -> usize {
        let slot = self.next_local_slot;
        self.next_local_slot += 1;
        if self.next_local_slot > self.max_local_slot {
            self.max_local_slot = self.next_local_slot;
        }

        slot
    }

    fn assign_storage(&mut self) -> Storage {
        if self.is_in_global_scope() {
            Storage::Global(self.get_global_slot())
        } else {
            Storage::Local(self.get_local_slot())
        }
    }
}
