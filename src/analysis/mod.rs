use crate::{
    ast::{
        AstView,
        expr::{ExprKind, ExprView, UniOp},
        idents::IdentId,
        item::{FunView, Item, ParamType},
        stmt::{StmtKind, StmtView},
        typ::{TypeKind, TypeView},
    },
    errors::Errors,
    tir::{
        self,
        def::DefId,
        typ::{AtomType, Type, TypeId},
    },
};
use std::collections::HashMap;

mod scopes;

pub struct Analysis {
    pub defs: tir::def::Defs,
    pub typs: tir::typ::Typs,
    pub ident_map: HashMap<IdentId, DefId>,
}

pub fn analyse(view: AstView, errors: &mut Errors) -> Analysis {
    let mut analyser = Analyser::new(view, errors);

    // first pass: declare types
    // todo: when ADTs are added

    // second pass: declare globals
    for item in view.items() {
        match &item.kind() {
            Item::Fun(fun) => {
                analyser.declare_fun(fun);
            }
        }
    }

    // third pass: analyse function bodies
    for item in view.items() {
        match &item.kind() {
            Item::Fun(fun) => {
                analyser.analyse_fun(fun);
            }
        }
    }

    Analysis {
        defs: analyser.defs,
        typs: analyser.typs,
        ident_map: analyser.ident_map,
    }
}

struct BlockState {
    ret_type: TypeId,
    loop_depth: u32,
}

struct Analyser<'a> {
    // view: AstView<'a>,
    typs: tir::typ::Typs,
    defs: tir::def::Defs,
    ident_map: HashMap<IdentId, DefId>,
    scopes: scopes::ScopeStack,
    errors: &'a mut Errors,
}
impl<'a> Analyser<'a> {
    pub fn new(view: AstView<'a>, errors: &'a mut Errors) -> Self {
        let mut scopes = scopes::ScopeStack::new();
        let typs = tir::typ::Typs::new();
        let defs = tir::def::Defs::new();
        let ident_map = HashMap::new();

        scopes.push_scope(); // built-in scope

        // Insert built-in types into the global scope
        for (string, atom) in ATOM_MAP {
            // only insert the type if the string is actually in the interner
            // current reason for this is that the interner is currently read-only
            if let Some(str_id) = view.interner().contains(string) {
                let typ = typs.atom(*atom);
                scopes.insert_typ(str_id, typ).unwrap();
            }
        }

        scopes.push_scope(); // file scope

        Self {
            // view,
            scopes,
            typs,
            defs,
            ident_map,
            errors,
        }
    }

    fn declare_fun(&mut self, fun: &FunView) {
        let mut params = Vec::new();
        let mut varargs = false;

        for (i, param) in fun.params.iter().enumerate() {
            match &param.typ {
                ParamType::Type(typ) => {
                    let typ = self.analyse_type(typ);
                    params.push(typ);
                }
                ParamType::Variadic(_) => {
                    if i != fun.params.len() - 1 {
                        self.errors.add(
                            "variadic parameter must be the last parameter",
                            param.ident.span(),
                        );
                        params.push(self.typs.error());
                    } else {
                        varargs = true;
                    }
                }
            }
        }

        let fun_typ = tir::typ::FunType {
            params,
            ret_type: fun
                .ret_type
                .as_ref()
                .map(|ret_type| self.analyse_type(ret_type))
                .unwrap_or(self.typs.atom(AtomType::Void)),
            varargs,
        };
        let typ = self.typs.fun(fun_typ);
        let def = self.defs.insert(fun.ident.str_id(), fun.ident.span(), typ);
        self.ident_map.insert(fun.ident.id(), def);
        if let Err(_prev) = self.scopes.insert_def(fun.ident.str_id(), def) {
            self.errors.add(
                format!("redefinition of function '{}'", fun.ident.str(),),
                fun.ident.span(),
            );
        }
    }
    fn analyse_fun(&mut self, fun: &FunView) {
        self.scopes.push_scope();

        let def_id = self.ident_map[&fun.ident.id()];
        let def = &self.defs[def_id];
        let typ = &self.typs[def.typ].as_fun().unwrap();
        let ret_type = typ.ret_type;

        for (param_type, param) in typ.params.iter().zip(fun.params.iter()) {
            let def = self
                .defs
                .insert(param.ident.str_id(), param.ident.span(), *param_type);
            self.ident_map.insert(param.ident.id(), def);

            if let Err(_prev) = self.scopes.insert_def(param.ident.str_id(), def) {
                self.errors.add(
                    format!("redefinition of parameter '{}'", param.ident.str(),),
                    param.ident.span(),
                );
            }
        }

        if let Some(stmts) = &fun.body {
            let mut block_state = BlockState {
                ret_type,
                loop_depth: 0,
            };
            self.analyse_block(stmts, &mut block_state);
        }

        self.scopes.pop_scope();
    }
    fn analyse_block(&mut self, stmts: &[StmtView], block_state: &mut BlockState) {
        for stmt in stmts {
            self.analyse_stmt(stmt, block_state);
        }
    }
    fn analyse_stmt(&mut self, stmt: &StmtView, block_state: &mut BlockState) {
        match &stmt.kind() {
            StmtKind::Expr(expr) => {
                self.analyse_expr(expr);
            }
            StmtKind::VarDecl { ident, expr, typ } => {
                let var_typ = typ.as_ref().map(|typ| self.analyse_type(typ));
                let expr_typ = self.analyse_expr(expr);
                if let Some(var_typ) = var_typ
                    && var_typ != expr_typ
                {
                    self.errors.add(
                        format!(
                            "expected type '{:?}' but got '{:?}'",
                            self.typs[var_typ], self.typs[expr_typ]
                        ),
                        ident.span(),
                    );
                }
                let def =
                    self.defs
                        .insert(ident.str_id(), ident.span(), var_typ.unwrap_or(expr_typ));
                let _ = self.scopes.insert_def(ident.str_id(), def); // allow shadowing
                self.ident_map.insert(ident.id(), def);
            }
            StmtKind::Assign { lhs, rhs } => {
                let lhs_type = self.analyse_expr(lhs);
                let rhs_type = self.analyse_expr(rhs);
                if lhs_type != rhs_type {
                    self.errors.add(
                        format!(
                            "type mismatch in assignment: expected '{:?}', got '{:?}'",
                            self.typs[lhs_type], self.typs[rhs_type]
                        ),
                        stmt.span(),
                    );
                }
            }
            StmtKind::Return(expr) => {
                let expr_type = expr
                    .as_ref()
                    .map(|expr| self.analyse_expr(expr))
                    .unwrap_or(self.typs.atom(AtomType::Void));
                if expr_type != block_state.ret_type {
                    self.errors.add(
                        format!(
                            "expected return type '{:?}' but got '{:?}'",
                            self.typs[block_state.ret_type], self.typs[expr_type]
                        ),
                        stmt.span(),
                    );
                }
            }
            StmtKind::If { elifs, els } => {
                for cond_block in elifs {
                    self.analyse_cond_block(cond_block, block_state, false);
                }
                if let Some(els) = els {
                    self.scopes.push_scope();
                    self.analyse_block(els, block_state);
                    self.scopes.pop_scope();
                }
            }
            StmtKind::While(cond_block) => {
                self.analyse_cond_block(cond_block, block_state, true);
            }
            StmtKind::Break => {
                if block_state.loop_depth == 0 {
                    self.errors
                        .add("cannot 'break' outside of a loop", stmt.span());
                }
            }
            StmtKind::Continue => {
                if block_state.loop_depth == 0 {
                    self.errors
                        .add("cannot 'continue' outside of a loop", stmt.span());
                }
            }
        }
    }
    fn analyse_cond_block(
        &mut self,
        cond_block: &super::ast::stmt::CondBlockView,
        block_state: &mut BlockState,
        is_loop: bool,
    ) {
        let cond_type = self.analyse_expr(&cond_block.cond);
        if cond_type != self.typs.atom(AtomType::Bool) {
            self.errors.add(
                format!("expected type 'bool' but got '{:?}'", self.typs[cond_type]),
                cond_block.cond.span(),
            );
        }
        self.scopes.push_scope();
        if is_loop {
            block_state.loop_depth += 1;
        }
        self.analyse_block(&cond_block.body, block_state);
        self.scopes.pop_scope();
        if is_loop {
            block_state.loop_depth -= 1;
        }
    }

    fn analyse_expr(&mut self, expr: &ExprView) -> TypeId {
        match &expr.kind() {
            ExprKind::Identifier(ident) => {
                if let Some(def_id) = self.scopes.get_def(ident.str_id()) {
                    self.ident_map.insert(ident.id(), def_id);
                    let def = &self.defs[def_id];
                    def.typ
                } else {
                    self.errors
                        .add(format!("undefined name '{}'", ident.str()), ident.span());
                    self.typs.error()
                }
            }
            ExprKind::Number(_) => self.typs.atom(AtomType::I32),
            ExprKind::String(_) => self.typs.ptr(self.typs.atom(AtomType::U8)),
            ExprKind::Group(expr) => self.analyse_expr(expr),
            ExprKind::BinOp { op, lhs, rhs } => {
                let lhs_type_id = self.analyse_expr(lhs);
                let rhs_type_id = self.analyse_expr(rhs);
                let lhs_type = &self.typs[lhs_type_id];
                let rhs_type = &self.typs[rhs_type_id];

                if lhs_type.is_error() || rhs_type.is_error() {
                    return self.typs.error();
                }

                if lhs_type_id != rhs_type_id {
                    self.errors.add(
                        format!(
                            "type mismatch in binary operation: left is '{:?}', right is '{:?}'",
                            lhs_type, rhs_type
                        ),
                        expr.span(),
                    );
                    return self.typs.error();
                }

                if op.is_comparison() {
                    self.typs.atom(tir::typ::AtomType::Bool)
                } else {
                    lhs_type_id
                }
            }
            ExprKind::UniOp { op, expr } => {
                let typ_id = self.analyse_expr(expr);
                let typ = &self.typs[typ_id];
                match op {
                    UniOp::Neg => match typ {
                        Type::Atom(atom_type)
                            if matches!(
                                atom_type,
                                tir::typ::AtomType::I8
                                    | tir::typ::AtomType::I16
                                    | tir::typ::AtomType::I32
                                    | tir::typ::AtomType::I64
                                    | tir::typ::AtomType::IPtr
                            ) =>
                        {
                            self.typs.atom(*atom_type)
                        }
                        Type::Error => self.typs.error(),
                        _ => {
                            self.errors.add(
                                format!("cannot apply the `-` operator on type '{:?}'", typ),
                                expr.span(),
                            );
                            self.typs.error()
                        }
                    },
                    UniOp::Not => match typ {
                        Type::Atom(atom_type)
                            if matches!(
                                atom_type,
                                tir::typ::AtomType::Bool
                                    | tir::typ::AtomType::U8
                                    | tir::typ::AtomType::U16
                                    | tir::typ::AtomType::U32
                                    | tir::typ::AtomType::U64
                                    | tir::typ::AtomType::UPtr
                                    | tir::typ::AtomType::I8
                                    | tir::typ::AtomType::I16
                                    | tir::typ::AtomType::I32
                                    | tir::typ::AtomType::I64
                                    | tir::typ::AtomType::IPtr
                            ) =>
                        {
                            self.typs.atom(*atom_type)
                        }
                        Type::Ptr(_) => {
                            // null check
                            self.typs.atom(tir::typ::AtomType::Bool)
                        }
                        Type::Error => self.typs.error(),
                        _ => {
                            self.errors.add(
                                format!("cannot apply the `!` operator on type '{:?}'", typ),
                                expr.span(),
                            );
                            self.typs.error()
                        }
                    },
                    UniOp::Ref => self.typs.ptr(typ_id),
                    UniOp::Deref => {
                        if let Type::Ptr(inner) = typ {
                            *inner
                        } else {
                            self.errors.add(
                                format!("cannot dereference type '{:?}'", self.typs[typ_id]),
                                expr.span(),
                            );
                            self.typs.error()
                        }
                    }
                }
            }
            ExprKind::Index { base, index } => {
                let base_type_id = self.analyse_expr(base);
                let index_type_id = self.analyse_expr(index);
                let base_type = &self.typs[base_type_id];
                let index_type = &self.typs[index_type_id];

                if index_type_id != self.typs.atom(AtomType::UPtr) {
                    self.errors.add(
                        format!("expected index of type 'uptr' but got '{:?}'", index_type),
                        index.span(),
                    );
                }

                match base_type {
                    Type::Ptr(inner) => *inner,
                    Type::Error => self.typs.error(),
                    _ => {
                        self.errors.add(
                            format!("cannot index into type '{:?}'", base_type),
                            base.span(),
                        );
                        self.typs.error()
                    }
                }
            }
            ExprKind::Call { callee, args } => {
                let callee_type_id = self.analyse_expr(callee);
                let args: Vec<_> = args.iter().map(|arg| self.analyse_expr(arg)).collect();
                let callee_type = &self.typs[callee_type_id];

                if let Type::Fun(fun_type) = callee_type {
                    if fun_type.params.len() > args.len() {
                        self.errors.add(
                            format!(
                                "too few arguments in function call: expected at least {}, got {}",
                                fun_type.params.len(),
                                args.len()
                            ),
                            expr.span(),
                        );
                    } else if !fun_type.varargs && fun_type.params.len() < args.len() {
                        self.errors.add(
                            format!(
                                "too many arguments in function call: expected {}, got {}",
                                fun_type.params.len(),
                                args.len()
                            ),
                            expr.span(),
                        );
                    }

                    for (i, (param_type, arg_type)) in
                        fun_type.params.iter().zip(args.iter()).enumerate()
                    {
                        if param_type != arg_type {
                            self.errors.add(
                                format!(
                                    "type mismatch in argument {}: expected '{:?}', got '{:?}'",
                                    i + 1,
                                    self.typs[*param_type],
                                    self.typs[*arg_type]
                                ),
                                expr.span(),
                            );
                        }
                    }

                    fun_type.ret_type
                } else if callee_type.is_error() {
                    self.typs.error()
                } else {
                    self.errors.add(
                        format!("cannot call value of type '{:?}'", callee_type),
                        callee.span(),
                    );
                    self.typs.error()
                }
            }
        }
    }
    fn analyse_type(&mut self, typ: &TypeView) -> tir::typ::TypeId {
        match &typ.kind() {
            TypeKind::Identifier(ident) => {
                if let Some(typ) = self.scopes.get_typ(ident.str_id()) {
                    typ
                } else {
                    self.errors
                        .add(format!("undefined type '{}'", ident.str()), ident.span());
                    self.typs.error()
                }
            }
            TypeKind::Ptr(inner) => {
                let inner_typ = self.analyse_type(inner);
                self.typs.ptr(inner_typ)
            }
        }
    }
}

const ATOM_MAP: &[(&str, tir::typ::AtomType)] = &[
    ("u8", tir::typ::AtomType::U8),
    ("u16", tir::typ::AtomType::U16),
    ("u32", tir::typ::AtomType::U32),
    ("u64", tir::typ::AtomType::U64),
    ("uptr", tir::typ::AtomType::UPtr),
    ("i8", tir::typ::AtomType::I8),
    ("i16", tir::typ::AtomType::I16),
    ("i32", tir::typ::AtomType::I32),
    ("i64", tir::typ::AtomType::I64),
    ("iptr", tir::typ::AtomType::IPtr),
    ("bool", tir::typ::AtomType::Bool),
    ("void", tir::typ::AtomType::Void),
];
