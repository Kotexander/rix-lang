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
        let printer = printer::AstPrinter::new(self);
        printer.fmt(f)
    }
}
