pub mod arena;
pub mod expr;
pub mod idents;
pub mod item;
pub mod literals;
mod printer;
pub mod stmt;
pub mod typ;

use expr::*;
use idents::*;
use item::*;
use literals::*;
use stmt::*;
use typ::*;

use crate::strings::Interner;

#[derive(Debug, Default)]
pub struct Ast {
    pub exprs: ExprArena,
    pub stmts: StmtArena,
    pub items: ItemArena,
    pub types: TypeArena,
    pub idents: IdentArena,
    pub strings: StrLitArena,
    pub numbers: NumLitArena,
}
impl Ast {
    pub fn view<'a>(&'a self, interner: &'a Interner) -> AstView<'a> {
        AstView::new(self, interner)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AstView<'a> {
    ast: &'a Ast,
    interner: &'a Interner,
}
impl<'a> AstView<'a> {
    pub fn new(ast: &'a Ast, interner: &'a Interner) -> Self {
        Self { ast, interner }
    }
    pub fn items(&self) -> impl Iterator<Item = ItemView<'a>> {
        (&self.ast.items)
            .into_iter()
            .map(|(id, _)| ItemView::new(*self, id))
    }

    pub fn ast(&self) -> &'a Ast {
        self.ast
    }

    pub fn interner(&self) -> &'a Interner {
        self.interner
    }
}
impl<'a> std::fmt::Display for AstView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (id, _) in &self.ast.items {
            let item_view = ItemView::new(*self, id);
            match item_view.kind() {
                Item::Fun(fun) => self.fmt_fun(f, &fun)?,
            }
        }
        Ok(())
    }
}
