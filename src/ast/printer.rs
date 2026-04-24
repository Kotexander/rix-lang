use super::{
    expr::{BinOp, ExprKind, ExprView, UniOp},
    item::{FunView, ParamType},
    stmt::{CondBlockView, StmtKind, StmtView},
    typ::{TypeKind, TypeView},
};

impl<'a> super::AstView<'a> {
    pub fn fmt_fun(&self, f: &mut std::fmt::Formatter<'_>, fun: &FunView) -> std::fmt::Result {
        write!(f, "fun {}(", fun.ident.str())?;
        for (i, param) in fun.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: ", param.ident.str())?;
            match &param.typ {
                ParamType::Type(typ) => {
                    self.fmt_type(f, typ)?;
                }
                ParamType::Variadic(_) => {
                    write!(f, "...")?;
                }
            }
        }
        write!(f, ")")?;
        if let Some(ret_type) = &fun.ret_type {
            write!(f, " : ")?;
            self.fmt_type(f, ret_type)?;
        } else {
            write!(f, " : void")?;
        }
        if let Some(body) = &fun.body {
            self.fmt_block(f, " ", 0, body)?;
            writeln!(f, "\n")?;
        } else {
            writeln!(f, ";\n")?;
        }
        Ok(())
    }
    pub fn fmt_type(&self, f: &mut std::fmt::Formatter<'_>, typ: &TypeView) -> std::fmt::Result {
        match &typ.kind() {
            TypeKind::Identifier(ident) => {
                let ident = ident.str();
                write!(f, "{ident}")
            }
            TypeKind::Ptr(inner) => {
                write!(f, "*")?;
                self.fmt_type(f, inner)
            }
        }
    }
    pub fn fmt_expr(&self, f: &mut std::fmt::Formatter<'_>, expr: &ExprView) -> std::fmt::Result {
        match &expr.kind() {
            ExprKind::Identifier(id) => {
                write!(f, "{}", id.str())
            }
            ExprKind::Number(id) => {
                write!(f, "{}", id.src())
            }
            ExprKind::String(id) => {
                write!(f, "{}", id.src())
            }
            ExprKind::Group(expr) => {
                write!(f, "(")?;
                self.fmt_expr(f, expr)?;
                write!(f, ")")
            }
            ExprKind::BinOp { op, lhs, rhs } => {
                self.fmt_expr(f, lhs)?;
                let op_str = binop_str(*op);
                write!(f, " {op_str} ")?;
                self.fmt_expr(f, rhs)
            }
            ExprKind::UniOp { op, expr } => {
                let op_str = uniop_str(*op);
                write!(f, "{op_str}")?;
                self.fmt_expr(f, expr)
            }
            ExprKind::Index { base, index } => {
                self.fmt_expr(f, base)?;
                write!(f, "[")?;
                self.fmt_expr(f, index)?;
                write!(f, "]")
            }
            ExprKind::Call { callee, args } => {
                self.fmt_expr(f, callee)?;
                write!(f, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    self.fmt_expr(f, arg)?;
                }
                write!(f, ")")
            }
        }
    }

    pub fn fmt_stmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        indents: u32,
        stmt: &StmtView,
    ) -> std::fmt::Result {
        for _ in 0..indents {
            write!(f, "    ")?;
        }
        match &stmt.kind() {
            StmtKind::Expr(expr) => {
                self.fmt_expr(f, expr)?;
                write!(f, ";")
            }
            StmtKind::Return(expr) => {
                write!(f, "return")?;
                if let Some(expr) = expr {
                    write!(f, " ")?;
                    self.fmt_expr(f, expr)?;
                }
                write!(f, ";")
            }
            StmtKind::VarDecl { ident, expr, typ } => {
                write!(f, "var {}", ident.str())?;
                if let Some(typ) = typ {
                    write!(f, " : ")?;
                    self.fmt_type(f, typ)?;
                }
                write!(f, " = ")?;
                self.fmt_expr(f, expr)?;
                write!(f, ";")
            }
            StmtKind::Assign { lhs, rhs } => {
                self.fmt_expr(f, lhs)?;
                write!(f, " = ")?;
                self.fmt_expr(f, rhs)?;
                write!(f, ";")
            }
            StmtKind::If { elifs, els } => {
                self.fmt_cond_block(f, "if", indents, &elifs[0])?;

                for cond_block in &elifs[1..] {
                    self.fmt_cond_block(f, " else if", indents, cond_block)?;
                }
                if let Some(els) = els {
                    self.fmt_block(f, " else ", indents, els)?;
                }
                Ok(())
            }
            StmtKind::While(cond_block) => self.fmt_cond_block(f, "while", indents, cond_block),
            StmtKind::Break => {
                write!(f, "break;")
            }
            StmtKind::Continue => {
                write!(f, "continue;")
            }
        }
    }
    fn fmt_cond_block(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        prefix: &str,
        indents: u32,
        cond_block: &CondBlockView,
    ) -> std::fmt::Result {
        write!(f, "{prefix} ")?;
        self.fmt_expr(f, &cond_block.cond)?;
        self.fmt_block(f, " ", indents, &cond_block.body)
    }
    fn fmt_block(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        prefix: &str,
        indents: u32,
        block: &[StmtView],
    ) -> std::fmt::Result {
        write!(f, "{prefix}{{")?;
        for stmt in block {
            writeln!(f)?;
            self.fmt_stmt(f, indents + 1, stmt)?;
        }
        writeln!(f)?;
        for _ in 0..indents {
            write!(f, "    ")?;
        }
        write!(f, "}}")
    }
}

fn binop_str(op: BinOp) -> &'static str {
    match op {
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
    }
}

fn uniop_str(op: UniOp) -> &'static str {
    match op {
        UniOp::Neg => "-",
        UniOp::Not => "!",
        UniOp::Ref => "&",
        UniOp::Deref => "*",
    }
}
