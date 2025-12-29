use super::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(pub(super) u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Identifier(Symbol),
    Integer(i64),
    String(Symbol),

    BinOp {
        left: ExprId,
        op: BinOp,
        right: ExprId,
    },
    UniOp {
        op: UniOp,
        expr: ExprId,
    },
    Index {
        base: ExprId,
        index: ExprId,
    },
    Call {
        callee: ExprId,
        args: Vec<ExprId>,
    },
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

    And,
    Xor,
    Or,

    LogicalAnd,
    LogicalOr,
}
impl BinOp {
    pub fn precedence(self) -> u8 {
        match self {
            BinOp::LogicalOr => 1,
            BinOp::LogicalAnd => 2,
            BinOp::Or => 3,
            BinOp::Xor => 4,
            BinOp::And => 5,
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
