pub mod arena;
pub mod expr;
pub mod idents;
pub mod item;
mod printer;
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
impl std::fmt::Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (_, item) in &self.items {
            match item {
                Item::Fun(fun) => printer::fmt_fun(f, self, fun)?,
            }
        }
        Ok(())
    }
}
