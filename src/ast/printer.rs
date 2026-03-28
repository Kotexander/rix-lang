use super::{
    Ast,
    expr::{BinOp, ExprId, ExprKind, UniOp},
    item::Fun,
    item::ParamType,
    stmt::{StmtId, StmtKind},
    typ::{TypeId, TypeKind},
};

pub fn fmt_fun(f: &mut std::fmt::Formatter<'_>, ast: &Ast, fun: &Fun) -> std::fmt::Result {
    let ident = ast.idents.str(fun.ident);
    write!(f, "fun {ident}(")?;
    for (i, param) in fun.params.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{}: ", ast.idents.str(param.ident))?;
        match param.typ {
            ParamType::Type(typ) => {
                fmt_type(f, ast, typ)?;
            }
            ParamType::Variadic(_) => {
                write!(f, "...")?;
            }
        }
    }
    write!(f, ")")?;
    if let Some(ret_type) = fun.ret_type {
        write!(f, " : ")?;
        fmt_type(f, ast, ret_type)?;
    } else {
        write!(f, " : void")?;
    }
    if let Some(body) = &fun.body {
        writeln!(f, " {{")?;
        for stmt in body {
            write!(f, "    ")?;
            fmt_stmt(f, ast, *stmt)?;
            writeln!(f)?;
        }
        writeln!(f, "}}\n")?;
    } else {
        writeln!(f, ";\n")?;
    }
    Ok(())
}
fn fmt_type(f: &mut std::fmt::Formatter<'_>, ast: &Ast, typ: TypeId) -> std::fmt::Result {
    let typ = &ast.types[typ];
    match typ.kind {
        TypeKind::Identifier(ident) => {
            let ident = ast.idents.str(ident);
            write!(f, "{ident}")
        }
        TypeKind::Ptr(inner) => {
            write!(f, "*")?;
            fmt_type(f, ast, inner)
        }
    }
}
fn fmt_expr(f: &mut std::fmt::Formatter<'_>, ast: &Ast, expr: ExprId) -> std::fmt::Result {
    let expr = &ast.exprs[expr];
    match &expr.kind {
        ExprKind::Identifier(id) => {
            write!(f, "{}", ast.idents.str(*id))
        }
        ExprKind::Integer(id) => {
            write!(f, "{}", ast.idents.str(*id))
        }
        ExprKind::String(id) => {
            write!(f, "{}", ast.idents.str(*id))
        }
        ExprKind::Group(expr) => {
            write!(f, "(")?;
            fmt_expr(f, ast, *expr)?;
            write!(f, ")")
        }
        ExprKind::BinOp { op, lhs, rhs } => {
            fmt_expr(f, ast, *lhs)?;
            let op_str = match op {
                BinOp::Mul => "*",
                BinOp::Div => "/",
                BinOp::Rem => "%",
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Shl => "<<",
                BinOp::Shr => ">>",
                BinOp::Lt => "<",
                BinOp::Gt => ">",
                BinOp::Le => "<=",
                BinOp::Ge => ">=",
                BinOp::Eq => "==",
                BinOp::Ne => "!=",
                BinOp::BitAnd => "&",
                BinOp::BitXor => "^",
                BinOp::BitOr => "|",
                BinOp::LogicalAnd => "and",
                BinOp::LogicalOr => "or",
            };
            write!(f, " {op_str} ")?;
            fmt_expr(f, ast, *rhs)
        }
        ExprKind::UniOp { op, expr } => {
            let op_str = match op {
                UniOp::Neg => "-",
                UniOp::Not => "!",
                UniOp::Ref => "&",
                UniOp::Deref => "*",
            };
            write!(f, "{op_str}")?;
            fmt_expr(f, ast, *expr)
        }
        ExprKind::Index { base, index } => {
            fmt_expr(f, ast, *base)?;
            write!(f, "[")?;
            fmt_expr(f, ast, *index)?;
            write!(f, "]")
        }
        ExprKind::Call { callee, args } => {
            fmt_expr(f, ast, *callee)?;
            write!(f, "(")?;
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                fmt_expr(f, ast, *arg)?;
            }
            write!(f, ")")
        }
    }
}
fn fmt_stmt(f: &mut std::fmt::Formatter<'_>, ast: &Ast, stmt: StmtId) -> std::fmt::Result {
    let stmt = &ast.stmts[stmt];
    match &stmt.kind {
        StmtKind::Expr(expr) => {
            fmt_expr(f, ast, *expr)?;
        }
        StmtKind::Return(expr) => {
            write!(f, "return")?;
            if let Some(expr) = expr {
                write!(f, " ")?;
                fmt_expr(f, ast, *expr)?;
            }
        }
        StmtKind::VarDecl { ident, expr, typ } => {
            write!(f, "var {}", ast.idents.str(*ident))?;
            if let Some(typ) = typ {
                write!(f, " : ")?;
                fmt_type(f, ast, *typ)?;
            }
            write!(f, " = ")?;
            fmt_expr(f, ast, *expr)?;
        }
    }
    write!(f, ";")
}
