use crate::{
    ast::{self, Ast},
    errors::Errors,
    lexer, tir,
};
use resolutions::*;
use scopes::*;

mod resolutions;
mod scopes;

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

struct BlockState {
    ret_type: tir::typ::TypeId,
    loop_depth: u32,
}
impl BlockState {
    fn new(ret_type: tir::typ::TypeId) -> Self {
        Self {
            ret_type,
            loop_depth: 0,
        }
    }
    fn enter_loop(&mut self) {
        self.loop_depth += 1;
    }
    fn exit_loop(&mut self) {
        self.loop_depth -= 1;
    }
    fn in_loop(&self) -> bool {
        self.loop_depth > 0
    }
}

struct Resolver<'a> {
    ast: &'a Ast,
    scope_stack: &'a mut ScopeStack,
    resolutions: &'a mut Resolutions,
    defs: &'a mut tir::def::Defs,
    typs: &'a mut tir::typ::Typs,
    errors: &'a mut Errors,
}
impl<'a> Resolver<'a> {
    fn resolve_type(&mut self, id: ast::typ::TypeId) -> tir::typ::TypeId {
        let typ = self.ast.types[id];
        match typ.kind {
            ast::typ::TypeKind::Identifier(ident) => {
                if let Some(id) = self.scope_stack.get_typ(self.ast.idents.str_id(ident)) {
                    id
                } else {
                    self.errors.add(
                        format!("unknown type '{}'", self.ast.idents.str(ident)),
                        self.ast.idents.span(ident),
                    );
                    self.typs.error()
                }
            }
            ast::typ::TypeKind::Ptr(id) => {
                let inner = self.resolve_type(id);
                self.typs.ptr(inner)
            }
        }
    }
    fn resolve_uni_op(
        &mut self,
        op: ast::expr::UniOp,
        expr: ast::expr::ExprId,
    ) -> tir::typ::TypeId {
        let typ_id = self.resolve_expr(expr);
        let typ = &self.typs[typ_id];
        match op {
            ast::expr::UniOp::Neg => match typ {
                tir::typ::Type::Atom(atom_type)
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
                tir::typ::Type::Error => self.typs.error(),
                _ => {
                    self.errors.add(
                        format!("cannot apply the `negation` operator on type '{:?}'", typ),
                        self.ast.exprs[expr].span,
                    );
                    self.typs.error()
                }
            },
            ast::expr::UniOp::Not => match typ {
                tir::typ::Type::Atom(atom_type)
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
                tir::typ::Type::Ptr(_) => {
                    // null check
                    self.typs.atom(tir::typ::AtomType::Bool)
                }
                tir::typ::Type::Error => self.typs.error(),
                _ => {
                    self.errors.add(
                        format!("cannot apply the `not` operator on type '{:?}'", typ),
                        self.ast.exprs[expr].span,
                    );
                    self.typs.error()
                }
            },
            ast::expr::UniOp::Ref => self.typs.ptr(typ_id),
            ast::expr::UniOp::Deref => {
                if let tir::typ::Type::Ptr(inner) = typ {
                    *inner
                } else {
                    self.errors.add(
                        format!("cannot dereference type '{:?}'", self.typs[typ_id]),
                        self.ast.exprs[expr].span,
                    );
                    self.typs.error()
                }
            }
        }
    }
    fn resolve_bin_op(
        &mut self,
        op: ast::expr::BinOp,
        lhs: ast::expr::ExprId,
        rhs: ast::expr::ExprId,
        span: lexer::Span,
    ) -> tir::typ::TypeId {
        let lhs_type_id = self.resolve_expr(lhs);
        let rhs_type_id = self.resolve_expr(rhs);
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
                span,
            );
            return self.typs.error();
        }

        if op.is_comparison() {
            self.typs.atom(tir::typ::AtomType::Bool)
        } else {
            lhs_type_id
        }
    }
    fn resolve_expr(&mut self, id: ast::expr::ExprId) -> tir::typ::TypeId {
        let expr = &self.ast.exprs[id];
        let typ = match &expr.kind {
            ast::expr::ExprKind::Identifier(ident_id) => {
                let ident_str_id = self.ast.idents.str_id(*ident_id);
                if let Some(def_id) = self.scope_stack.get_def(ident_str_id) {
                    let def = &self.defs[def_id];
                    def.typ
                } else {
                    self.errors.add(
                        format!("undefined name '{}'", self.ast.idents.str(*ident_id)),
                        self.ast.idents.span(*ident_id),
                    );
                    self.typs.error()
                }
            }
            ast::expr::ExprKind::Integer(_ident_id) => self.typs.atom(tir::typ::AtomType::I32),
            ast::expr::ExprKind::String(_ident_id) => {
                self.typs.ptr(self.typs.atom(tir::typ::AtomType::U8))
            }
            ast::expr::ExprKind::Group(expr) => self.resolve_expr(*expr),
            ast::expr::ExprKind::BinOp { op, lhs, rhs } => {
                self.resolve_bin_op(*op, *lhs, *rhs, expr.span)
            }
            ast::expr::ExprKind::UniOp { op, expr } => self.resolve_uni_op(*op, *expr),
            ast::expr::ExprKind::Index { base, index } => {
                let base_type_id = self.resolve_expr(*base);
                let index_type_id = self.resolve_expr(*index);
                let base_type = &self.typs[base_type_id];
                let index_type = &self.typs[index_type_id];
                let inner_type = match base_type {
                    tir::typ::Type::Ptr(inner) => *inner,
                    tir::typ::Type::Error => self.typs.error(),
                    _ => {
                        self.errors.add(
                            format!("cannot index into type '{:?}'", self.typs[base_type_id]),
                            self.ast.exprs[*base].span,
                        );
                        self.typs.error()
                    }
                };

                if index_type_id != self.typs.atom(tir::typ::AtomType::UPtr) {
                    self.errors.add(
                        format!(
                            "type mismatch in index expression: expected '{:?}', got '{:?}'",
                            tir::typ::AtomType::UPtr,
                            index_type
                        ),
                        self.ast.exprs[*index].span,
                    );
                }

                inner_type
            }
            ast::expr::ExprKind::Call { callee, args } => {
                let callee_type_id = self.resolve_expr(*callee);

                let resolved_args: Vec<_> =
                    args.iter().map(|arg| self.resolve_expr(*arg)).collect();

                let callee_type = &self.typs[callee_type_id];
                match callee_type {
                    tir::typ::Type::Fun(fun_type) => {
                        if fun_type.params.len() > resolved_args.len() {
                            self.errors.add(
                                format!(
                                    "too few arguments in function call: expected at least {}, got {}",
                                    fun_type.params.len(),
                                    resolved_args.len()
                                ),
                                expr.span,
                            );
                        } else if !fun_type.varargs && fun_type.params.len() < resolved_args.len() {
                            self.errors.add(
                                format!(
                                    "too many arguments in function call: expected {}, got {}",
                                    fun_type.params.len(),
                                    resolved_args.len()
                                ),
                                expr.span,
                            );
                        }
                        for (i, param_type) in fun_type.params.iter().enumerate() {
                            let arg_type = resolved_args[i];
                            // report error if all of the following are true:
                            // type mismatch
                            // argument type is not already an error
                            // parameter type is not already an error
                            if *param_type != arg_type
                            // && !self.typs[arg_type].is_error()
                            // && !self.typs[*param_type].is_error()
                            {
                                self.errors.add(
                                    format!(
                                        "type mismatch in argument {}: expected '{:?}', got '{:?}'",
                                        i + 1,
                                        self.typs[*param_type],
                                        self.typs[arg_type]
                                    ),
                                    expr.span,
                                );
                            }
                        }

                        fun_type.ret_type
                    }
                    tir::typ::Type::Error => self.typs.error(),
                    _ => {
                        self.errors.add(
                            format!("cannot call non-function type '{:?}'", callee_type),
                            self.ast.exprs[*callee].span,
                        );
                        self.typs.error()
                    }
                }
            }
        };
        self.resolutions.expr_types.insert(id, typ);
        typ
    }

    fn resolve_block(&mut self, block: &[ast::stmt::StmtId], state: &mut BlockState) {
        for stmt_id in block {
            let stmt = &self.ast.stmts[*stmt_id];
            match &stmt.kind {
                ast::stmt::StmtKind::Expr(expr) => {
                    self.resolve_expr(*expr);
                }
                ast::stmt::StmtKind::VarDecl { ident, expr, typ } => {
                    let resolved_typ = typ.map(|id| self.resolve_type(id));
                    let expr_typ = self.resolve_expr(*expr);
                    if let Some(resolved_typ) = resolved_typ
                        && resolved_typ != expr_typ
                    {
                        self.errors.add(
                                format!(
                                    "type mismatch in variable declaration: expected '{:?}', got '{:?}'",
                                    self.typs[resolved_typ],
                                    self.typs[expr_typ]),
                                self.ast.idents.span(*ident)
                            );
                    }
                    let var_typ = resolved_typ.unwrap_or(expr_typ);
                    let def_id = self.defs.insert(*ident, var_typ);

                    // we allow shadowing
                    let _ = self
                        .scope_stack
                        .insert_def(self.ast.idents.str_id(*ident), def_id);
                }
                ast::stmt::StmtKind::Assign { lhs, rhs } => {
                    let lhs_type_id = self.resolve_expr(*lhs);
                    let rhs_type_id = self.resolve_expr(*rhs);
                    let lhs_type = &self.typs[lhs_type_id];
                    let rhs_type = &self.typs[rhs_type_id];

                    if lhs_type_id != rhs_type_id && !(lhs_type.is_error() || rhs_type.is_error()) {
                        self.errors.add(
                            format!(
                                "type mismatch in assignment: expected '{:?}', got '{:?}'",
                                self.typs[lhs_type_id], self.typs[rhs_type_id]
                            ),
                            stmt.span,
                        );
                    }
                }
                ast::stmt::StmtKind::Return(expr) => {
                    let expr_typ = expr
                        .map(|expr| self.resolve_expr(expr))
                        .unwrap_or(self.typs.atom(tir::typ::AtomType::Void));
                    if expr_typ != state.ret_type {
                        self.errors.add(
                            format!(
                                "type mismatch in return statement: expected '{:?}', got '{:?}'",
                                self.typs[state.ret_type], self.typs[expr_typ]
                            ),
                            stmt.span,
                        );
                    }
                }
                ast::stmt::StmtKind::If { elifs, els } => {
                    for cond_block in elifs {
                        self.resolve_cond_block(cond_block, state, false);
                    }
                    if let Some(els) = els {
                        self.scope_stack.push_scope();
                        self.resolve_block(els, state);
                        self.scope_stack.pop_scope();
                    }
                }
                ast::stmt::StmtKind::While(cond_block) => {
                    self.resolve_cond_block(cond_block, state, true);
                }
                ast::stmt::StmtKind::Break => {
                    if !state.in_loop() {
                        self.errors
                            .add("break statement not inside a loop", stmt.span);
                    }
                }
                ast::stmt::StmtKind::Continue => {
                    if !state.in_loop() {
                        self.errors
                            .add("continue statement not inside a loop", stmt.span);
                    }
                }
            }
        }
    }
    fn resolve_cond_block(
        &mut self,
        cond_block: &ast::stmt::CondBlock,
        state: &mut BlockState,
        is_loop: bool,
    ) {
        let cond_type = self.resolve_expr(cond_block.cond);
        if cond_type != self.typs.atom(tir::typ::AtomType::Bool) {
            self.errors.add(
                format!(
                    "type mismatch in condition: expected '{:?}', got '{:?}'",
                    tir::typ::AtomType::Bool,
                    self.typs[cond_type]
                ),
                self.ast.exprs[cond_block.cond].span,
            );
        }
        self.scope_stack.push_scope();
        if is_loop {
            state.enter_loop();
        }
        self.resolve_block(&cond_block.body, state);
        if is_loop {
            state.exit_loop();
        }
        self.scope_stack.pop_scope();
    }

    fn declare_fun(&mut self, fun: &ast::item::Fun) -> Result<(), tir::def::DefId> {
        // resolve parameter types
        let mut params = vec![];
        let mut varargs = false;
        for (i, param) in fun.params.iter().enumerate() {
            match param.typ {
                ast::item::ParamType::Type(typ) => params.push(self.resolve_type(typ)),
                ast::item::ParamType::Variadic(_) => {
                    if i != fun.params.len() - 1 {
                        self.errors.add(
                            "variadic parameter must be the last parameter".to_string(),
                            self.ast.idents.span(param.ident),
                        );
                        params.push(self.typs.error());
                    } else {
                        varargs = true;
                        // TODO: have a proper type for variadic parameters instead of this placeholder
                        // params.push(self.typs.error());
                    }
                }
            }
        }
        // resolve return type
        let ret_type = fun
            .ret_type
            .map(|typ| self.resolve_type(typ))
            .unwrap_or_else(|| self.typs.atom(tir::typ::AtomType::Void));

        let fun_type = tir::typ::FunType {
            params,
            ret_type,
            varargs,
        };
        let fun_type_id = self.typs.fun(fun_type);
        let def_id = self.defs.insert(fun.ident, fun_type_id);
        self.resolutions.def_resolutions.insert(fun.ident, def_id);
        self.scope_stack
            .insert_def(self.ast.idents.str_id(fun.ident), def_id)
    }
    fn resolve_fun(&mut self, fun: &ast::item::Fun) {
        let def_id = self.resolutions.def_resolutions[&fun.ident];
        let fun_type_id = self.defs[def_id].typ;
        let fun_type = &self.typs[fun_type_id].as_fun().unwrap();

        if fun.body.is_some() && fun_type.varargs {
            self.errors.add(
                "variadic functions cannot have bodies for now",
                self.ast.idents.span(fun.ident),
            );
            return;
        }
        self.scope_stack.push_scope(); // function body scope

        for (i, param_type) in fun_type.params.iter().enumerate() {
            let param = fun.params[i];
            let param_def_id = self.defs.insert(param.ident, *param_type);

            if let Err(_old_def_id) = self
                .scope_stack
                .insert_def(self.ast.idents.str_id(param.ident), param_def_id)
            {
                self.errors.add(
                    format!(
                        "duplicate parameter name '{}'",
                        self.ast.idents.str(param.ident)
                    ),
                    self.ast.idents.span(param.ident),
                );
            }
        }

        if let Some(body) = &fun.body {
            let mut state = BlockState::new(fun_type.ret_type);
            self.resolve_block(body, &mut state);
        }

        self.scope_stack.pop_scope(); // function body scope
    }
}

pub fn analyze(ast: &Ast, errors: &mut Errors) {
    let mut scope_stack = ScopeStack::new();
    let mut defs = tir::def::Defs::new();
    let mut typs = tir::typ::Typs::new();
    let mut resolutions = Resolutions::new();
    let mut resolver = Resolver {
        ast,
        scope_stack: &mut scope_stack,
        resolutions: &mut resolutions,
        defs: &mut defs,
        typs: &mut typs,
        errors,
    };

    resolver.scope_stack.push_scope(); // built-in scope

    // insert primitive types into the built-in scope
    // we only insert the primitive type if that string actually appeares
    // in the AST since its too late to mutate it by interning new strings
    // should be fine for now but can be changed later
    for (prim, str_id) in ATOM_MAP
        .iter()
        .filter_map(|(str, prim)| ast.idents.check(str).map(|str_id| (prim, str_id)))
    {
        let typ_id = resolver.typs.atom(*prim);
        resolver.scope_stack.insert_typ(str_id, typ_id).unwrap();
    }

    resolver.scope_stack.push_scope(); // file scope

    // first pass: collect type definitions
    // TODO: when ADTs get added

    // second pass: resolve types of globals
    for (_, item) in &ast.items {
        match item {
            ast::item::Item::Fun(fun) => {
                if let Err(_def_id) = resolver.declare_fun(fun) {
                    resolver.errors.add(
                        format!(
                            "duplicate definition of '{}'",
                            resolver.ast.idents.str(fun.ident)
                        ),
                        resolver.ast.idents.span(fun.ident),
                    );
                }
            }
        }
    }

    // third pass: resolve function bodies
    for (_, item) in &ast.items {
        match item {
            ast::item::Item::Fun(fun) => {
                resolver.resolve_fun(fun);
            }
        }
    }

    scope_stack.push_scope(); // file scope
    scope_stack.push_scope(); // built-in scope
}
