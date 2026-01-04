pub mod expr;
pub mod item;
pub mod stmt;
pub mod typ;

use expr::*;
use item::*;
use stmt::*;
use typ::*;

use crate::{
    ast::typ::{Type, TypeId},
    lexer,
};

#[derive(Debug, Default)]
pub struct Ast {
    exprs: Vec<Expr>,
    stmts: Vec<Stmt>,
    items: Vec<Item>,
    types: Vec<Type>,
    symbols: Interner,
}
impl Ast {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_expr(&mut self, kind: ExprKind, span: lexer::Span) -> ExprId {
        let id = self.exprs.len() as u32;
        self.exprs.push(Expr::new(kind, span));
        ExprId(id)
    }
    pub fn get_expr(&self, id: ExprId) -> &Expr {
        &self.exprs[id.0 as usize]
    }
    pub fn add_type(&mut self, kind: TypeKind, span: lexer::Span) -> TypeId {
        let id = self.types.len() as u32;
        self.types.push(Type::new(kind, span));
        TypeId(id)
    }
    pub fn get_type(&self, id: TypeId) -> &Type {
        &self.types[id.0 as usize]
    }
    pub fn add_stmt(&mut self, stmt: Stmt) -> StmtId {
        let id = self.stmts.len() as u32;
        self.stmts.push(stmt);
        StmtId(id)
    }
    pub fn get_stmt(&self, id: StmtId) -> &Stmt {
        &self.stmts[id.0 as usize]
    }
    pub fn add_item(&mut self, item: Item) -> ItemId {
        let id = self.items.len() as u32;
        self.items.push(item);
        ItemId(id)
    }
    pub fn get_item(&self, id: ItemId) -> &Item {
        &self.items[id.0 as usize]
    }

    pub fn get_symbol(&mut self, s: &str) -> Symbol {
        self.symbols.intern(s)
    }
    pub fn resolve_symbol(&self, symbol: Symbol) -> &str {
        self.symbols.resolve(symbol)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(u32);

/// A symbol with its original span information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniqueSymbol {
    pub sym: Symbol,
    pub span: lexer::Span,
}
impl UniqueSymbol {
    pub fn new(sym: Symbol, span: lexer::Span) -> Self {
        Self { sym, span }
    }
}

#[derive(Debug, Default)]
struct Interner {
    strings: Vec<String>,
}
impl Interner {
    pub fn intern(&mut self, s: &str) -> Symbol {
        if let Some(pos) = self.strings.iter().position(|x| x == s) {
            Symbol(pos as u32)
        } else {
            self.strings.push(s.into());
            Symbol((self.strings.len() - 1) as u32)
        }
    }

    pub fn resolve(&self, symbol: Symbol) -> &str {
        self.strings[symbol.0 as usize].as_str()
    }
}
