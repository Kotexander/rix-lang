pub mod expr;

use expr::*;

#[derive(Debug, Default)]
pub struct Ast {
    exprs: Vec<Expr>,
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

    pub fn intern_symbol(&mut self, s: &str) -> Symbol {
        self.symbols.intern(s)
    }
    pub fn get_symbol(&self, symbol: Symbol) -> &str {
        self.symbols.resolve(symbol)
    }

    pub fn display_expr<'a>(&'a self, expr_id: ExprId) -> ExprDisplay<'a> {
        ExprDisplay::new(self, expr_id)
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
