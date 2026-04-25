use crate::{
    analysis, ast,
    strings::{Interner, StrId},
};
use std::num::NonZeroU32;

pub mod def;
pub mod typ;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Temp(NonZeroU32);
struct TempBuilder {
    tmp: u32,
}
impl TempBuilder {
    fn new() -> Self {
        Self { tmp: 1 }
    }
    fn inc(&mut self) -> Temp {
        let tmp = Temp(unsafe { NonZeroU32::new_unchecked(self.tmp) });
        self.tmp += 1;
        tmp
    }
}

#[derive(Debug)]
pub enum Value {
    Tmp(Temp),
    Def(def::DefId),
    ConstInt(u32),
    Str(StrId),
}
impl From<Temp> for Value {
    fn from(value: Temp) -> Self {
        Self::Tmp(value)
    }
}
impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::ConstInt(value)
    }
}
impl From<def::DefId> for Value {
    fn from(value: def::DefId) -> Self {
        Self::Def(value)
    }
}
impl From<StrId> for Value {
    fn from(value: StrId) -> Self {
        Self::Str(value)
    }
}

#[derive(Debug)]
pub enum Instr {
    // Alloca {
    //     dst: def::DefId,
    //     typ: typ::TypeId,
    // },
    Load {
        dst: Temp,
        src: def::DefId,
    },
    Store {
        dst: def::DefId,
        src: Value,
    },
    Return {
        val: Option<Value>,
    },
    Jump {
        blk: u32,
    },
    JumpCond {
        cond: Value,
        then_blk: u32,
        else_blk: u32,
    },
    Cmp {
        dst: Temp,
        lhs: Value,
        rhs: Value,
        op: CmpOp,
    },
    BinOp {
        dst: Temp,
        lhs: Value,
        rhs: Value,
        op: BinOp,
    },
    // UniOp {
    //     dst: Temp,
    //     operand: Value,
    //     op: ast::expr::UniOp,
    // },
    Call {
        dst: Temp,
        callee: Value,
        args: Vec<Value>,
    },
    Index {
        dst: Temp,
        base: Value,
        index: Value,
        typ: typ::TypeId,
    },
}
#[derive(Debug)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}
#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
pub struct Block {
    pub name: String,
    pub instrs: Vec<Instr>,
}
impl Block {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            instrs: Vec::new(),
        }
    }

    // fn build_alloca(&mut self, dst: def::DefId, typ: typ::TypeId) {
    //     self.instrs.push(Instr::Alloca { dst, typ });
    // }
    fn build_load(&mut self, dst: Temp, src: def::DefId) {
        self.instrs.push(Instr::Load { dst, src });
    }
    fn build_store(&mut self, dst: def::DefId, src: impl Into<Value>) {
        self.instrs.push(Instr::Store {
            dst,
            src: src.into(),
        });
    }
    fn build_return(&mut self, val: Option<impl Into<Value>>) {
        self.instrs.push(Instr::Return {
            val: val.map(Into::into),
        });
    }
    fn build_jump(&mut self, blk: u32) {
        self.instrs.push(Instr::Jump { blk });
    }
    fn build_jump_cond(&mut self, cond: impl Into<Value>, then_blk: u32, else_blk: u32) {
        self.instrs.push(Instr::JumpCond {
            cond: cond.into(),
            then_blk,
            else_blk,
        });
    }
    fn build_cmp(&mut self, dst: Temp, lhs: impl Into<Value>, rhs: impl Into<Value>, op: CmpOp) {
        self.instrs.push(Instr::Cmp {
            dst,
            lhs: lhs.into(),
            rhs: rhs.into(),
            op,
        });
    }
    fn build_binop(&mut self, dst: Temp, lhs: impl Into<Value>, rhs: impl Into<Value>, op: BinOp) {
        self.instrs.push(Instr::BinOp {
            dst,
            lhs: lhs.into(),
            rhs: rhs.into(),
            op,
        });
    }
    fn build_call(&mut self, dst: Temp, callee: impl Into<Value>, args: Vec<Value>) {
        self.instrs.push(Instr::Call {
            dst,
            callee: callee.into(),
            args,
        });
    }
    fn build_index(
        &mut self,
        dst: Temp,
        base: impl Into<Value>,
        index: impl Into<Value>,
        typ: typ::TypeId,
    ) {
        self.instrs.push(Instr::Index {
            dst,
            base: base.into(),
            index: index.into(),
            typ,
        });
    }
}

#[derive(Debug)]
pub struct Fun {
    pub def: def::DefId,
    pub args: Vec<def::DefId>,
    pub body: Option<FunBody>,
}
#[derive(Debug)]
pub struct FunBody {
    pub locals: Vec<def::DefId>,
    pub blocks: Vec<Block>,
}
impl FunBody {
    pub fn new_block(&mut self, name: impl Into<String>) -> u32 {
        let blk = self.blocks.len() as u32;
        self.blocks.push(Block::new(name));
        blk
    }
}
struct FunBodyBuilder<'a> {
    analysis: &'a analysis::Analysis,
    fun: FunBody,
    tmp: TempBuilder,
}
impl<'a> FunBodyBuilder<'a> {
    fn new(analysis: &'a analysis::Analysis) -> Self {
        Self {
            analysis,
            fun: FunBody {
                locals: Vec::new(),
                blocks: vec![Block::new("entry")],
            },
            tmp: TempBuilder::new(),
        }
    }
    fn build_stmt(
        &mut self,
        stmt: &ast::stmt::StmtView,
        blk: u32,
        break_blk: Option<u32>,
        continue_blk: Option<u32>,
    ) -> u32 {
        match &stmt.kind() {
            ast::stmt::StmtKind::Expr(expr) => {
                self.build_expr(expr, blk);
                blk
            }
            ast::stmt::StmtKind::VarDecl { ident, expr, .. } => {
                let def = self.analysis.names[&ident.id()];
                let val = self.build_expr(expr, blk);
                let block = &mut self.fun.blocks[blk as usize];
                block.build_store(def, val);
                self.fun.locals.push(def);
                blk
            }
            ast::stmt::StmtKind::Assign { lhs, rhs } => {
                if let ast::expr::ExprKind::Identifier(ident) = &lhs.kind() {
                    let def = self.analysis.names[&ident.id()];
                    let val = self.build_expr(rhs, blk);
                    let block = &mut self.fun.blocks[blk as usize];
                    block.build_store(def, val);
                    blk
                } else {
                    panic!("invalid assignment target");
                }
            }
            ast::stmt::StmtKind::Return(expr) => {
                let val = expr.as_ref().map(|e| self.build_expr(e, blk));
                let block = &mut self.fun.blocks[blk as usize];
                block.build_return(val);
                blk
            }
            ast::stmt::StmtKind::If { elifs, els } => {
                let mut blks = vec![];
                let then_blk = self.fun.new_block("if.then");
                for cond_block in &elifs[1..] {
                    let cond_blk = self.fun.new_block("if.else.cond");
                    let then_blk = self.fun.new_block("if.else.then");
                    blks.push((cond_blk, then_blk, cond_block));
                }

                let else_block = if els.is_some() {
                    Some(self.fun.new_block("if.else"))
                } else {
                    None
                };

                let end_blk = self.fun.new_block("if.end");
                let else_blk = else_block.unwrap_or(end_blk);

                self.build_cond(
                    blk,
                    then_blk,
                    // first cond or else or end
                    blks.first().map(|elif| elif.0).unwrap_or(else_blk),
                    end_blk,
                    &elifs[0],
                    break_blk,
                    continue_blk,
                );
                for i in 0..blks.len() {
                    self.build_cond(
                        blks[i].0,
                        blks[i].1,
                        // next cond or else or end
                        blks.get(i + 1).map(|elif| elif.0).unwrap_or(else_blk),
                        end_blk,
                        blks[i].2,
                        break_blk,
                        continue_blk,
                    );
                }
                if let Some(els) = els {
                    for stmt in els {
                        self.build_stmt(stmt, else_blk, break_blk, continue_blk);
                    }
                    let else_block = &mut self.fun.blocks[else_blk as usize];
                    else_block.build_jump(end_blk);
                }
                end_blk
            }
            ast::stmt::StmtKind::While(cond_block) => {
                let cond_blk = self.fun.new_block("while.cond");
                let body_blk = self.fun.new_block("while.body");
                let end_blk = self.fun.new_block("while.end");

                let block = &mut self.fun.blocks[blk as usize];
                block.build_jump(cond_blk);

                self.build_cond(
                    cond_blk,
                    body_blk,
                    end_blk,
                    cond_blk,
                    cond_block,
                    Some(end_blk),
                    Some(cond_blk),
                );

                end_blk
            }
            ast::stmt::StmtKind::Break => {
                let block = &mut self.fun.blocks[blk as usize];
                block.build_jump(break_blk.unwrap());
                blk
            }
            ast::stmt::StmtKind::Continue => {
                let block = &mut self.fun.blocks[blk as usize];
                block.build_jump(continue_blk.unwrap());
                blk
            }
        }
    }
    fn build_expr(&mut self, expr: &ast::expr::ExprView, blk: u32) -> Value {
        match &expr.kind() {
            ast::expr::ExprKind::Identifier(ident) => {
                let block = &mut self.fun.blocks[blk as usize];
                let tmp = self.tmp.inc();
                block.build_load(tmp, self.analysis.names[&ident.id()]);
                tmp.into()
            }
            ast::expr::ExprKind::Number(num) => {
                let num: u32 = num.num().try_into().unwrap();
                num.into()
            }
            ast::expr::ExprKind::String(str) => Value::Str(str.str_id()),
            ast::expr::ExprKind::Group(expr) => self.build_expr(expr, blk),
            ast::expr::ExprKind::BinOp { op, lhs, rhs } if op.is_comparison() => {
                let lhs_val = self.build_expr(lhs, blk);
                let rhs_val = self.build_expr(rhs, blk);
                let cmp_op = match op {
                    ast::expr::BinOp::Eq => CmpOp::Eq,
                    ast::expr::BinOp::Ne => CmpOp::Ne,
                    ast::expr::BinOp::Lt => CmpOp::Lt,
                    ast::expr::BinOp::Gt => CmpOp::Gt,
                    ast::expr::BinOp::Le => CmpOp::Le,
                    ast::expr::BinOp::Ge => CmpOp::Ge,
                    _ => unreachable!(),
                };

                let block = &mut self.fun.blocks[blk as usize];
                let tmp = self.tmp.inc();
                block.build_cmp(tmp, lhs_val, rhs_val, cmp_op);
                tmp.into()
            }
            ast::expr::ExprKind::BinOp { op, lhs, rhs } => {
                let lhs_val = self.build_expr(lhs, blk);
                let rhs_val = self.build_expr(rhs, blk);
                let bin_op = match op {
                    ast::expr::BinOp::Add => BinOp::Add,
                    ast::expr::BinOp::Sub => BinOp::Sub,
                    ast::expr::BinOp::Mul => BinOp::Mul,
                    ast::expr::BinOp::Div => BinOp::Div,
                    _ => unimplemented!(),
                };

                let block = &mut self.fun.blocks[blk as usize];
                let tmp = self.tmp.inc();
                block.build_binop(tmp, lhs_val, rhs_val, bin_op);
                tmp.into()
            }
            ast::expr::ExprKind::UniOp { .. } => todo!(),
            ast::expr::ExprKind::Index { base, index } => {
                let ast::expr::ExprKind::Identifier(ident) = base.kind() else {
                    panic!();
                };
                let base_id = self.analysis.names[&ident.id()];
                let base_val = self.build_expr(base, blk);
                // let base_val = self.build_expr(base, blk);

                let index_val = self.build_expr(index, blk);
                let typ = self.analysis.defs[base_id].typ;

                let block = &mut self.fun.blocks[blk as usize];
                let tmp = self.tmp.inc();
                block.build_index(tmp, base_val, index_val, typ);
                tmp.into()
            }
            ast::expr::ExprKind::Call { callee, args } => {
                let ast::expr::ExprKind::Identifier(ident) = callee.kind() else {
                    panic!();
                };
                let callee_val = self.analysis.names[&ident.id()];

                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.build_expr(arg, blk));
                }

                let block = &mut self.fun.blocks[blk as usize];
                let tmp = self.tmp.inc();
                block.build_call(tmp, callee_val, arg_vals);
                tmp.into()
            }
        }
    }
    pub fn build_cond(
        &mut self,
        blk: u32,
        then: u32,
        els: u32,
        end: u32,
        cond_block: &ast::stmt::CondBlockView,
        break_blk: Option<u32>,
        continue_blk: Option<u32>,
    ) {
        let cond = self.build_expr(&cond_block.cond, blk);

        let block = &mut self.fun.blocks[blk as usize];
        block.build_jump_cond(cond, then, els);

        for stmt in &cond_block.body {
            self.build_stmt(stmt, then, break_blk, continue_blk);
        }
        let then_block = &mut self.fun.blocks[then as usize];
        then_block.build_jump(end);
    }

    // fn new_block(&mut self, name: impl Into<String>) -> u32 {
    //     let blk = self.blocks.len() as u32;
    //     self.blocks.push(Block::new(name));
    //     blk
    // }
}

#[derive(Debug)]
pub struct Tir {
    pub typs: typ::Typs,
    pub defs: def::Defs,
    pub funs: Vec<Fun>,
}
impl Tir {
    pub fn view<'a>(&'a self, interner: &'a Interner) -> TirView<'a> {
        TirView {
            tir: self,
            interner,
        }
    }
}

pub fn lower(ast: ast::AstView, analysis: analysis::Analysis) -> Tir {
    let mut funs = vec![];

    for item in ast.items() {
        match &item.kind() {
            ast::item::Item::Fun(fun) => {
                funs.push(lower_fun(&analysis, fun));
            }
        }
    }

    Tir {
        typs: analysis.typs,
        defs: analysis.defs,
        funs,
    }
}

fn lower_fun(analysis: &analysis::Analysis, fun: &ast::item::FunView) -> Fun {
    let fun_def_id = analysis.names[&fun.ident.id()];

    let args: Vec<_> = fun
        .params
        .iter()
        .filter_map(|param| {
            if let ast::item::ParamType::Type(_) = param.typ {
                Some(param.ident.id())
            } else {
                None
            }
        })
        .map(|ident| analysis.names[&ident])
        .collect();

    if fun.body.is_none() {
        return Fun {
            def: fun_def_id,
            args,
            body: None,
        };
    }

    let mut builder = FunBodyBuilder::new(analysis);
    let mut blk = 0; // entry block
    for stmt in fun.body.as_ref().unwrap() {
        blk = builder.build_stmt(stmt, blk, None, None);
    }

    Fun {
        def: fun_def_id,
        args,
        body: Some(builder.fun),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TirView<'a> {
    pub tir: &'a Tir,
    pub interner: &'a Interner,
}
impl<'a> std::fmt::Display for TirView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for fun in &self.tir.funs {
            writeln!(
                f,
                "fun {} {{",
                self.interner.resolve(self.tir.defs[fun.def].str)
            )?;
            if let Some(body) = &fun.body {
                // print locals
                if !body.locals.is_empty() {
                    writeln!(f, "  // locals")?;
                    for local in &body.locals {
                        writeln!(
                            f,
                            "  ({:?}){}",
                            local,
                            self.interner.resolve(self.tir.defs[*local].str),
                        )?;
                    }
                }

                for (i, blk) in body.blocks.iter().enumerate() {
                    writeln!(f, "  ({i}) {}:", blk.name)?;
                    for instr in &blk.instrs {
                        writeln!(f, "        {:?}", instr)?;
                    }
                }
            }
            writeln!(f, "}}")?;
        }
        Ok(())
    }
}
