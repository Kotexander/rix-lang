pub mod expr;
pub mod stmt;

use expr::*;
use stmt::*;

#[derive(Debug, Default)]
pub struct Ast {
    exprs: Vec<Expr>,
    stmts: Vec<Stmt>,
    symbols: Interner,
}
impl Ast {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_expr(&mut self, expr: Expr) -> ExprId {
        let id = self.exprs.len() as u32;
        self.exprs.push(expr);
        ExprId(id)
    }
    pub fn get_expr(&self, id: ExprId) -> &Expr {
        &self.exprs[id.0 as usize]
    }

    pub fn get_symbol(&mut self, s: &str) -> Symbol {
        self.symbols.intern(s)
    }
    pub fn resolve_symbol(&self, symbol: Symbol) -> &str {
        self.symbols.resolve(symbol)
    }

    pub fn add_stmt(&mut self, stmt: Stmt) -> StmtId {
        let id = self.stmts.len() as u32;
        self.stmts.push(stmt);
        StmtId(id)
    }
    pub fn get_stmt(&self, id: StmtId) -> &Stmt {
        &self.stmts[id.0 as usize]
    }

    pub fn display_expr<'a>(&'a self, expr_id: ExprId) -> ExprDisplay<'a> {
        ExprDisplay::new(self, expr_id)
    }

    pub fn stmts(&self) -> &[Stmt] {
        &self.stmts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(u32);

#[derive(Debug, Default)]
struct Interner {
    strings: Vec<String>,
}
impl Interner {
    // fn new() -> Self {
    //     Self::default()
    // }

    fn intern(&mut self, s: &str) -> Symbol {
        if let Some(pos) = self.strings.iter().position(|x| x == s) {
            Symbol(pos as u32)
        } else {
            self.strings.push(s.into());
            Symbol((self.strings.len() - 1) as u32)
        }
    }

    fn resolve(&self, symbol: Symbol) -> &str {
        self.strings[symbol.0 as usize].as_str()
    }
}
