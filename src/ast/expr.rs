use super::Symbol;
use crate::lexer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(pub(super) u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Identifier(Symbol),
    Integer(i64),
    String(Symbol),

    Group(ExprId),
    BinOp { op: BinOp, lhs: ExprId, rhs: ExprId },
    UniOp { op: UniOp, expr: ExprId },
    Index { base: ExprId, index: ExprId },
    Call { callee: ExprId, args: Vec<ExprId> },

    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: lexer::Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Mul,
    Div,
    Rem,

    Add,
    Sub,

    Shl,
    Shr,

    Lt,
    Gt,
    Le,
    Ge,

    Eq,
    Ne,

    BitAnd,
    BitXor,
    BitOr,

    LogicalAnd,
    LogicalOr,
}
impl BinOp {
    pub fn precedence(self) -> u8 {
        match self {
            BinOp::LogicalOr => 1,
            BinOp::LogicalAnd => 2,
            BinOp::BitOr => 3,
            BinOp::BitXor => 4,
            BinOp::BitAnd => 5,
            BinOp::Eq | BinOp::Ne => 6,
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => 7,
            BinOp::Shl | BinOp::Shr => 8,
            BinOp::Add | BinOp::Sub => 9,
            BinOp::Mul | BinOp::Div | BinOp::Rem => 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniOp {
    Neg,
    Not,
    Ref,
    Deref,
}

#[derive(Clone, Copy)]
pub struct ExprDisplay<'a> {
    ast: &'a super::Ast,
    expr_id: ExprId,
    indent: usize,
    last: bool,
}
impl<'a> ExprDisplay<'a> {
    pub fn new(ast: &'a super::Ast, expr_id: ExprId) -> Self {
        Self {
            ast,
            expr_id,
            indent: 0,
            last: true,
        }
    }
    fn indent(self, expr_id: ExprId, last: bool) -> Self {
        Self {
            ast: self.ast,
            expr_id,
            indent: self.indent + 1,
            last,
        }
    }
}
impl std::fmt::Display for ExprDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _i in 0..self.indent {
            write!(f, "   ")?;
        }
        if self.indent > 0 {
            if self.last {
                write!(f, "└─ ")?;
            } else {
                write!(f, "├─ ")?;
            }
        }
        let expr = self.ast.get_expr(self.expr_id);
        match &expr.kind {
            ExprKind::Identifier(symbol) => {
                let name = self.ast.get_symbol(*symbol);
                writeln!(f, "{}", name)
            }
            ExprKind::Integer(value) => {
                writeln!(f, "{}", value)
            }
            ExprKind::String(symbol) => {
                let s = self.ast.get_symbol(*symbol);
                writeln!(f, "{:?}", s)
            }
            ExprKind::BinOp { op, lhs, rhs } => {
                writeln!(f, "({:?})", op)?;
                write!(f, "{}", self.indent(*lhs, false))?;
                write!(f, "{}", self.indent(*rhs, true))
            }
            ExprKind::Group(expr_id) => {
                writeln!(f, "(group)")?;
                write!(f, "{}", self.indent(*expr_id, true))
            }
            ExprKind::UniOp { op, expr } => {
                writeln!(f, "({:?})", op)?;
                write!(f, "{}", self.indent(*expr, true))
            }
            ExprKind::Index { base, index } => {
                writeln!(f, "(Index)")?;
                write!(f, "{}", self.indent(*base, false))?;
                write!(f, "{}", self.indent(*index, true))
            }
            ExprKind::Call { callee, args } => {
                writeln!(f, "(Call)")?;
                write!(f, "{}", self.indent(*callee, args.is_empty()))?;
                for (i, arg) in args.iter().enumerate() {
                    write!(f, "{}", self.indent(*arg, i == args.len() - 1))?;
                }
                Ok(())
            }
            ExprKind::Error => writeln!(f, "<expr>"),
        }
    }
}
