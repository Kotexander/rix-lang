use crate::{
    ast::{
        AstView,
        expr::{ExprKindView, ExprView},
        idents::IdentView,
        item::{FunView, ItemKindView},
        stmt::{StmtKindView, StmtView},
        typ::{TypeKindView, TypeView},
    },
    errors::Errors,
    strings::Interner,
    tir::{
        def::{Def, DefId, Defs},
        typ::{TypId, Typs},
    },
};
pub use resolutions::NameResolutions;
use scopes::ScopeStack;

mod resolutions;
mod scopes;

pub struct ScopeResolver {
    resolutions: NameResolutions,
    scopes: ScopeStack,
}
impl ScopeResolver {
    pub fn new(num_idents: usize) -> Self {
        Self {
            resolutions: NameResolutions::new(num_idents),
            scopes: ScopeStack::new(),
        }
    }

    pub fn new_def(&mut self, ident: IdentView, def_id: DefId) -> Result<(), DefId> {
        self.resolutions.set_def(ident.id(), def_id);
        self.scopes.insert_def(ident.str_id(), def_id)
    }
    pub fn new_typ(&mut self, ident: IdentView, typ_id: TypId) -> Result<(), TypId> {
        self.resolutions.set_typ(ident.id(), typ_id);
        self.scopes.insert_typ(ident.str_id(), typ_id)
    }

    pub fn resolve_def<F>(&mut self, ident: IdentView, err: F) -> Option<DefId>
    where
        F: FnOnce(),
    {
        let Some(def) = self.scopes.get_def(ident.str_id()) else {
            err();
            return None;
        };
        self.resolutions.set_def(ident.id(), def);
        Some(def)
    }
    pub fn resolve_typ<F>(&mut self, ident: IdentView, err: F) -> TypId
    where
        F: FnOnce() -> TypId,
    {
        let typ = self.scopes.get_typ(ident.str_id()).unwrap_or_else(err);
        self.resolutions.set_typ(ident.id(), typ);
        typ
    }
}

struct State<'a> {
    scope_resolver: &'a mut ScopeResolver,
    defs: &'a mut Defs,
    typs: &'a mut Typs,
    errors: &'a mut Errors,
}

impl<'a> State<'a> {
    fn declare_builtin_types(&mut self, interner: &Interner) {
        for (str, atom) in super::ATOM_MAP {
            let Some(str_id) = interner.contains(str) else {
                continue;
            };
            let typ_id = self.typs.atom(*atom);
            self.scope_resolver
                .scopes
                .insert_typ(str_id, typ_id)
                .unwrap();
        }
    }
    fn get_type(&mut self, typ: TypeView) -> TypId {
        match typ.kind() {
            TypeKindView::Identifier(ident) => self.scope_resolver.resolve_typ(ident, || {
                self.errors
                    .add(format!("unknown type `{}`", ident.str()), ident.span());
                self.typs.error()
            }),
            TypeKindView::Ptr(inner) => {
                let inner = self.get_type(inner);
                self.typs.ptr(inner)
            }
        }
    }
    fn declare_fun(&mut self, fun: FunView) -> DefId {
        let def = self.defs.alloc(Def {
            ident: fun.ident().id(),
        });
        if let Err(prev) = self.scope_resolver.new_def(fun.ident(), def) {
            self.errors.add(
                format!("duplicate function definition of `{}`", fun.ident().str()),
                fun.ident().span(),
            );
            return prev;
        }
        def
    }
    fn resolve_fun(&mut self, fun: FunView) {
        self.scope_resolver.scopes.push_scope();

        for param in fun.params() {
            let def = self.defs.alloc(Def {
                ident: param.ident().id(),
            });
            if let Err(_prev) = self.scope_resolver.new_def(param.ident(), def) {
                self.errors.add(
                    format!("duplicate parameter name of `{}`", param.ident().str()),
                    param.ident().span(),
                );
            }
        }

        if let Some(body) = fun.body() {
            for stmt in body.iter() {
                self.resolve_stmt(stmt);
            }
        }

        self.scope_resolver.scopes.pop_scope();
    }
    fn resolve_stmt(&mut self, stmt: StmtView) {
        match stmt.kind() {
            StmtKindView::Expr(expr_view) => self.resolve_expr(expr_view),
            StmtKindView::VarDecl {
                ident,
                expr,
                typ: _,
            } => {
                // resolve expr first
                self.resolve_expr(expr);

                let def = self.defs.alloc(Def { ident: ident.id() });
                // allow shadowing
                let _ = self.scope_resolver.new_def(ident, def);
            }
            StmtKindView::Assign { lhs, rhs } => {
                self.resolve_expr(lhs);
                self.resolve_expr(rhs);
            }
            StmtKindView::Return(expr) => {
                if let Some(expr) = expr {
                    self.resolve_expr(expr);
                }
            }
            StmtKindView::If { elifs, els } => {
                for cond_block in elifs.iter() {
                    self.resolve_expr(cond_block.cond());
                    self.scope_resolver.scopes.push_scope();
                    for stmt in cond_block.block().iter() {
                        self.resolve_stmt(stmt);
                    }
                    self.scope_resolver.scopes.pop_scope();
                }
                if let Some(els) = els {
                    self.scope_resolver.scopes.push_scope();
                    for stmt in els.iter() {
                        self.resolve_stmt(stmt);
                    }
                    self.scope_resolver.scopes.pop_scope();
                }
            }
            StmtKindView::While(cond_block_view) => {
                self.resolve_expr(cond_block_view.cond());
                self.scope_resolver.scopes.push_scope();
                for stmt in cond_block_view.block().iter() {
                    self.resolve_stmt(stmt);
                }
                self.scope_resolver.scopes.pop_scope();
            }
            StmtKindView::Break => {}
            StmtKindView::Continue => {}
        }
    }
    fn resolve_expr(&mut self, expr: ExprView) {
        match expr.kind() {
            ExprKindView::Identifier(ident) => {
                self.scope_resolver.resolve_def(ident, || {
                    self.errors.add(
                        format!("unknown identifier `{}`", ident.str()),
                        ident.span(),
                    );
                });
            }
            ExprKindView::Number(_) => {}
            ExprKindView::String(_) => {}
            ExprKindView::Group(expr) => self.resolve_expr(expr),
            ExprKindView::BinOp { op: _, lhs, rhs } => {
                self.resolve_expr(lhs);
                self.resolve_expr(rhs);
            }
            ExprKindView::UniOp { op: _, expr } => {
                self.resolve_expr(expr);
            }
            ExprKindView::Index { base, index } => {
                self.resolve_expr(base);
                self.resolve_expr(index);
            }
            ExprKindView::Call { callee, args } => {
                self.resolve_expr(callee);
                for arg in args.iter() {
                    self.resolve_expr(arg);
                }
            }
        }
    }
}

pub fn resolve_names(
    view: AstView,
    defs: &mut Defs,
    typs: &mut Typs,
    errors: &mut Errors,
) -> NameResolutions {
    let mut scope_resolver = ScopeResolver::new(view.ast().idents.len());
    let mut state = State {
        scope_resolver: &mut scope_resolver,
        defs,
        typs,
        errors,
    };

    state.scope_resolver.scopes.push_scope(); // built-in scope
    state.declare_builtin_types(view.interner());

    state.scope_resolver.scopes.push_scope(); // file/global scope

    // first pass: declare items
    for item in view.items() {
        match item.kind() {
            ItemKindView::Fun(fun) => {
                state.declare_fun(fun);
            }
        }
    }

    // second pass: resolve items
    for item in view.items() {
        match item.kind() {
            ItemKindView::Fun(fun) => {
                state.resolve_fun(fun);
            }
        }
    }

    state.scope_resolver.scopes.pop_scope(); // file/global scope
    state.scope_resolver.scopes.pop_scope(); // built-in scope

    #[cfg(debug_assertions)]
    if !state.scope_resolver.scopes.is_empty() {
        eprintln!("warning: some scopes were not closed properly");
    }

    scope_resolver.resolutions
}
