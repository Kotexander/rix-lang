pub mod arena;
pub mod expr;
pub mod idents;
pub mod item;
pub mod stmt;
pub mod typ;

use expr::*;
use idents::*;
use item::*;
use stmt::*;
use typ::*;

#[derive(Debug, Default)]
pub struct Ast {
    pub exprs: ExprArena,
    pub stmts: StmtArena,
    pub items: ItemArena,
    pub types: TypeArena,
    pub idents: Idents,
}
