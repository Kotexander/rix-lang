pub mod arena;
pub mod expr;
pub mod item;
pub mod stmt;
pub mod symbols;
pub mod typ;

use expr::*;
use item::*;
use stmt::*;
use symbols::*;
use typ::*;

#[derive(Debug, Default)]
pub struct Ast {
    pub exprs: ExprArena,
    pub args: ArgListArena,
    pub stmts: StmtArena,
    pub items: ItemArena,
    pub types: TypeArena,
    pub symbols: Symbols,
}
