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

use crate::{arena::Arena, strings::Interner};

#[derive(Debug, Default)]
pub struct Ast {
    pub exprs: Arena<Expr>,
    // for calls and array literals
    pub exprs_lists: Arena<ExprId>,

    pub types: Arena<Type>,

    pub stmts: Arena<Stmt>,
    // this represents blocks
    pub stmts_lists: Arena<StmtId>,
    // for `if` and `while` conditions
    pub cond_blocks: Arena<CondBlock>,
    // for `if else` chains
    pub cond_blocks_lists: Arena<CondBlockId>,

    pub items: Arena<Item>,
    pub params: Arena<Param>,

    pub idents: Arena<Ident>,
    pub strings: Arena<StrLit>,
    pub numbers: Arena<NumLit>,
}
impl Ast {
    pub fn view<'a>(&'a self, interner: &'a Interner, src: &'a str) -> AstView<'a> {
        AstView::new(self, interner, src)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AstView<'a> {
    ast: &'a Ast,
    interner: &'a Interner,
    src: &'a str,
}
impl<'a> AstView<'a> {
    pub fn new(ast: &'a Ast, interner: &'a Interner, src: &'a str) -> Self {
        Self { ast, interner, src }
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

    pub fn src(&self) -> &'a str {
        self.src
    }
}
impl<'a> std::fmt::Display for AstView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for fun in self.items() {
            match &fun.kind() {
                ItemKindView::Fun(fun) => {
                    self.fmt_fun(f, fun)?;
                }
            }
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! define_view {
    (
        $view_name:ident,   // e.g., NumLitView
        $node_name:ident,   // e.g., NumLit (used for the helper function)
        $id_type:ident,     // e.g., NumLitId
        $arena_field:ident  // e.g., numbers (the field inside ast)
    ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $view_name<'a> {
            view: super::AstView<'a>,
            id: $id_type,
        }

        impl<'a> $view_name<'a> {
            #[inline]
            pub fn new(view: super::AstView<'a>, id: $id_type) -> Self {
                Self { view, id }
            }

            #[inline]
            pub fn id(&self) -> $id_type {
                self.id
            }

            #[inline]
            pub fn span(&self) -> lexer::Span {
                self.node().span
            }

            #[inline]
            fn node(&self) -> &'a $node_name {
                &self.view.ast.$arena_field[self.id]
            }
        }
    };
}
