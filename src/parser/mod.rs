use crate::{
    ast::{
        Ast,
        expr::{BinOp, ExprId, ExprKind, UniOp},
        idents::IdentId,
        item::{Fun, Item, ItemId, Param, ParamType},
        stmt::{StmtId, StmtKind},
        typ::{TypeId, TypeKind},
    },
    errors::Errors,
    lexer::{LexerWindow, Span, Tok, Token},
};

pub struct Parser<'a> {
    lexer: LexerWindow<'a>,
    ast: Ast,
    errors: &'a mut Errors,
}
impl<'a> Parser<'a> {
    pub fn new(input: &'a str, errors: &'a mut Errors) -> Self {
        Parser {
            lexer: LexerWindow::new(input),
            ast: Ast::default(),
            errors,
        }
    }
    pub fn finish(self) -> Ast {
        self.ast
    }

    fn slice_ident(&mut self, span: Span) -> IdentId {
        let ident = self.lexer.slice(span);
        self.ast.idents.add(ident, span)
    }

    fn expect_tokens(&mut self, expected: &[Tok]) -> Result<Token, ()> {
        let peek = *self.lexer.peek();
        if expected.contains(&peek.kind) {
            Ok(self.lexer.advance())
        } else {
            self.errors.add(expect_msg(expected, &peek.kind), peek.span);
            Err(())
        }
    }

    pub fn parse(&mut self) {
        while self.lexer.peek().kind != Tok::Eof {
            if self.parse_item().is_err() {
                self.find_next_item();
            }
        }
    }
}
// Type
impl<'input> Parser<'input> {
    /// Type
    ///     = identifier
    ///     | "*" Type
    pub fn parse_type(&mut self) -> Result<TypeId, ()> {
        let peek = self.lexer.peek();
        match peek.kind {
            Tok::Identifier => {
                let identifier = self.lexer.advance();
                let ident = self.slice_ident(identifier.span);

                Ok(self
                    .ast
                    .types
                    .add(TypeKind::Identifier(ident), identifier.span))
            }
            Tok::Asterisk => {
                let asterisk = self.lexer.advance().span;
                let inner_type = self.parse_type()?;
                let span = asterisk.join(&self.ast.types[inner_type].span);
                Ok(self.ast.types.add(TypeKind::Ptr(inner_type), span))
            }
            _ => {
                self.errors
                    .add(format!("expected type, but found {}", peek.kind), peek.span);
                Err(())
            }
        }
    }
}
// Expr
impl<'input> Parser<'input> {
    pub fn parse_expr(&mut self) -> Result<ExprId, ()> {
        self.parse_binop_expr(0)
    }

    /// Term
    ///     = number
    ///     | identifier
    ///     | string
    ///     | "(" Expr ")"
    pub fn parse_term(&mut self) -> Result<ExprId, ()> {
        let peek = self.lexer.peek();
        match peek.kind {
            Tok::Number => {
                let number = self.lexer.advance();
                let kind = ExprKind::Integer(self.slice_ident(number.span));
                Ok(self.ast.exprs.add(kind, number.span))
            }
            Tok::Identifier => {
                let identifer = self.lexer.advance();
                let ident = self.slice_ident(identifer.span);
                Ok(self
                    .ast
                    .exprs
                    .add(ExprKind::Identifier(ident), identifer.span))
            }
            Tok::String => {
                let string = self.lexer.advance();
                let ident = self.slice_ident(string.span);
                Ok(self.ast.exprs.add(ExprKind::String(ident), string.span))
            }
            Tok::LParen => {
                let lparen = self.lexer.advance().span;
                let expr = self.parse_expr()?;
                let rparen = self.expect_tokens(&[Tok::RParen])?.span;
                Ok(self
                    .ast
                    .exprs
                    .add(ExprKind::Group(expr), lparen.join(&rparen)))
            }
            _ => {
                let peek = *peek;
                self.errors.add(
                    format!("expected an expression, but found {}", peek.kind),
                    peek.span,
                );
                Err(())
            }
        }
    }
    /// SuffixExpr
    ///     = Term (Index | FnCallArgs)*
    ///
    /// Index
    ///     = "\[" Expr "\]"
    ///
    /// FnCallArgs
    ///     = "(" (Expr ",")* Expr? ")"
    fn parse_suffix_expr(&mut self) -> Result<ExprId, ()> {
        let mut expr = self.parse_term()?;

        loop {
            let peek = self.lexer.peek();
            match peek.kind {
                Tok::LBracket => {
                    let lbracket = self.lexer.advance().span;
                    let index = self.parse_expr()?;
                    let rbracket = self.expect_tokens(&[Tok::RBracket])?.span;
                    expr = self.ast.exprs.add(
                        ExprKind::Index { base: expr, index },
                        self.ast.exprs[expr].span.join(&lbracket).join(&rbracket),
                    );
                }
                Tok::LParen => {
                    self.lexer.advance(); // lparen

                    let mut args = Vec::new();
                    while self.lexer.peek().kind != Tok::RParen {
                        let arg = self.parse_expr()?;
                        args.push(arg);

                        let peek = self.lexer.peek();
                        match peek.kind {
                            Tok::RParen => break,
                            Tok::Comma => {
                                self.lexer.advance();
                            }
                            Tok::RBrace | Tok::RBracket | Tok::Colon | Tok::Eof => {
                                let peek = *self.lexer.peek();
                                self.errors.add(
                                    expect_msg(&[Tok::Comma, Tok::RParen], &peek.kind),
                                    peek.span,
                                );
                                return Err(());
                            }
                            _ => {
                                // assume there was a missing comma and try to continue parsing
                                self.errors.add(
                                    format!("expected {} after argument", Tok::Comma),
                                    peek.span,
                                );
                            }
                        }
                    }
                    // next token should be rparen
                    let rparen = self.lexer.advance().span;
                    expr = self.ast.exprs.add(
                        ExprKind::Call { callee: expr, args },
                        self.ast.exprs[expr].span.join(&rparen),
                    );
                }
                _ => break,
            }
        }
        Ok(expr)
    }
    /// PrefixExpr
    ///     = PrefixOp* SuffixExpr
    ///
    /// PrefixOp
    ///     = "-" | "!" | "&" | "*"
    fn parse_prefix_expr(&mut self) -> Result<ExprId, ()> {
        let peek = self.lexer.peek();
        let op = match peek.kind {
            Tok::Minus => UniOp::Neg,
            Tok::Bang => UniOp::Not,
            Tok::Ampersand => UniOp::Ref,
            Tok::Asterisk => UniOp::Deref,
            _ => return self.parse_suffix_expr(),
        };
        let op_token = self.lexer.advance();
        let expr = self.parse_prefix_expr()?;
        let span = op_token.span.join(&self.ast.exprs[expr].span);
        Ok(self.ast.exprs.add(ExprKind::UniOp { op, expr }, span))
    }

    fn parse_binop_expr(&mut self, min_prec: u8) -> Result<ExprId, ()> {
        let mut lhs = self.parse_prefix_expr()?;

        loop {
            let peek = self.lexer.peek();

            // check if the next token is a binary operator
            let Some(op) = bin_op_from_tok(&peek.kind) else {
                break;
            };

            // check operator precedence
            let prec = bin_op_precedence(op);
            if prec < min_prec {
                break;
            }
            let _op_token = self.lexer.advance();

            let Ok(rhs) = self.parse_binop_expr(prec + 1) else {
                return Ok(lhs);
            };

            let lhs_span = self.ast.exprs[lhs].span;
            let rhs_span = self.ast.exprs[rhs].span;
            let span = lhs_span.join(&rhs_span);

            lhs = self.ast.exprs.add(ExprKind::BinOp { op, lhs, rhs }, span);
        }

        Ok(lhs)
    }
}
// Stmt
impl<'input> Parser<'input> {
    /// VarDecl
    ///     = "var" identifier (":" Type)? "=" Expr ";"
    pub fn parse_var(&mut self) -> Result<StmtId, ()> {
        let var = self.expect_tokens(&[Tok::Var])?;

        let ident_tok = self.expect_tokens(&[Tok::Identifier])?;
        let typ = if self.lexer.match_(Tok::Colon).is_ok() {
            Some(self.parse_type()?)
        } else {
            None
        };
        let _equal = self.expect_tokens(&[Tok::Equal])?;
        let expr = self.parse_expr()?;
        let semicolon = self.expect_tokens(&[Tok::Semicolon])?;

        let ident = self.slice_ident(ident_tok.span);
        let span = var.span.join(&semicolon.span);
        Ok(self
            .ast
            .stmts
            .add(StmtKind::VarDecl { ident, expr, typ }, span))
    }
    /// Stmt
    ///    = VarDecl
    ///    | Expr ";"
    ///    | Return (Expr)? ";"
    pub fn parse_stmt(&mut self) -> Result<StmtId, ()> {
        let peek = self.lexer.peek();
        match peek.kind {
            Tok::Var => self.parse_var(),
            Tok::Return => {
                let return_ = self.lexer.advance();
                let expr = if self.lexer.peek().kind != Tok::Semicolon {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                let semicolon = self.expect_tokens(&[Tok::Semicolon])?;
                let span = return_.span.join(&semicolon.span);
                Ok(self.ast.stmts.add(StmtKind::Return(expr), span))
            }
            _ => {
                let expr = self.parse_expr()?;
                let semicolon = self.expect_tokens(&[Tok::Semicolon])?;
                let span = self.ast.exprs[expr].span.join(&semicolon.span);
                Ok(self.ast.stmts.add(StmtKind::Expr(expr), span))
            }
        }
    }
    fn find_next_stmt(&mut self) {
        let mut level = 0u32;
        loop {
            let peek = self.lexer.peek();
            match peek.kind {
                Tok::LBrace => {
                    self.lexer.advance();
                    level += 1
                }
                Tok::RBrace => {
                    self.lexer.advance();
                    if level == 0 {
                        break;
                    }
                    level -= 1;
                }
                Tok::Semicolon => {
                    self.lexer.advance();
                    if level == 0 {
                        break;
                    }
                }
                Tok::Eof => break,
                _ => {
                    self.lexer.advance();
                }
            }
        }
    }
    /// Block
    ///     = "{" Stmt* "}"
    pub fn parse_block(&mut self) -> Result<Vec<StmtId>, ()> {
        let _lbrace = self.expect_tokens(&[Tok::LBrace]);
        let mut stmts = vec![];

        while self.lexer.peek().kind != Tok::Eof {
            let stmt = self.parse_stmt();
            if let Ok(stmt) = stmt {
                stmts.push(stmt);
            } else {
                self.find_next_stmt();
            }
            if matches!(self.lexer.peek().kind, Tok::RBrace | Tok::Eof) {
                break;
            }
        }
        let _rbrace = self.expect_tokens(&[Tok::RBrace])?;
        Ok(stmts)
    }
}
// Item
impl<'input> Parser<'input> {
    /// ParamList
    ///     = "(" (identifier ":" Type ",")* (identifier ":" Type)? ")"
    fn parse_param_list(&mut self) -> Result<Vec<Param>, ()> {
        let _lparen = self.expect_tokens(&[Tok::LParen])?;
        let mut params = Vec::new();

        while self.lexer.peek().kind != Tok::RParen {
            let ident_tok = self.expect_tokens(&[Tok::Identifier])?;
            let _colon = self.expect_tokens(&[Tok::Colon])?;

            let typ = if let Ok(dotdotdot) = self.lexer.match_(Tok::DotDotDot) {
                ParamType::Variadic(dotdotdot)
            } else {
                ParamType::Type(self.parse_type()?)
            };

            let ident = self.slice_ident(ident_tok.span);
            params.push(Param::new(typ, ident));

            let peek = self.lexer.peek();
            match peek.kind {
                Tok::RParen => break,
                Tok::Comma => {
                    self.lexer.advance();
                }
                Tok::Identifier => {
                    // assume there was a missing comma and try to continue parsing
                    self.errors.add(
                        format!("expected {} after parameter", Tok::Comma),
                        peek.span,
                    );
                }
                _ => {
                    self.errors.add(
                        expect_msg(&[Tok::Comma, Tok::RParen], &peek.kind),
                        peek.span,
                    );
                    return Err(());
                }
            }
        }
        // next token should be rparen
        let _rparen = self.lexer.advance();

        Ok(params)
    }
    /// Fun
    ///     = "fun" identifier ParamList (":" Type)? (Block | ";")
    pub fn parse_fun(&mut self) -> Result<ItemId, ()> {
        let _fun = self.expect_tokens(&[Tok::Fun])?;
        let ident_tok = self.expect_tokens(&[Tok::Identifier])?;
        let params = self.parse_param_list()?;
        let ret_type = if let Ok(_colon) = self.lexer.match_(Tok::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = if let Ok(_semicolon) = self.lexer.match_(Tok::Semicolon) {
            None
        } else {
            Some(self.parse_block()?)
        };
        let ident = self.slice_ident(ident_tok.span);
        Ok(self.ast.items.alloc(Item::Fun(Fun {
            ident,
            params,
            ret_type,
            body,
        })))
    }

    pub fn parse_item(&mut self) -> Result<ItemId, ()> {
        let peek = self.lexer.peek();
        match peek.kind {
            Tok::Fun => self.parse_fun(),
            _ => {
                let peek = *peek;
                self.errors.add(
                    format!("expected an item, but found {}", peek.kind),
                    peek.span,
                );
                Err(())
            }
        }
    }

    fn find_next_item(&mut self) {
        let mut level = 0u32;
        loop {
            let peek = self.lexer.peek();
            match peek.kind {
                Tok::Fun => {
                    if level == 0 {
                        break;
                    }
                }
                Tok::LBrace => {
                    self.lexer.advance();
                    level += 1
                }
                Tok::RBrace => {
                    self.lexer.advance();
                    if level == 0 {
                        break;
                    }
                    level -= 1;
                }
                Tok::Eof => break,
                _ => {
                    self.lexer.advance();
                }
            }
        }
    }
}

fn bin_op_from_tok(tok: &Tok) -> Option<BinOp> {
    match tok {
        Tok::Plus => Some(BinOp::Add),
        Tok::Minus => Some(BinOp::Sub),
        Tok::Asterisk => Some(BinOp::Mul),
        Tok::Slash => Some(BinOp::Div),
        Tok::Percent => Some(BinOp::Rem),
        Tok::EqualEqual => Some(BinOp::Eq),
        Tok::BangEqual => Some(BinOp::Ne),
        Tok::Less => Some(BinOp::Lt),
        Tok::LessEqual => Some(BinOp::Le),
        Tok::Greater => Some(BinOp::Gt),
        Tok::GreaterEqual => Some(BinOp::Ge),
        Tok::And => Some(BinOp::LogicalAnd),
        Tok::Or => Some(BinOp::LogicalOr),
        Tok::Ampersand => Some(BinOp::BitAnd),
        Tok::Caret => Some(BinOp::BitXor),
        Tok::Pipe => Some(BinOp::BitOr),
        Tok::LessLess => Some(BinOp::Shl),
        Tok::GreaterGreater => Some(BinOp::Shr),
        _ => None,
    }
}

fn bin_op_precedence(op: BinOp) -> u8 {
    match op {
        BinOp::LogicalOr => 1,
        BinOp::LogicalAnd => 2,
        BinOp::BitOr => 3,
        BinOp::BitXor => 4,
        BinOp::BitAnd => 5,
        BinOp::Eq | BinOp::Ne => 6,
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => 7,
        BinOp::Shl | BinOp::Shr => 8,
        BinOp::Add | BinOp::Sub => 9,
        BinOp::Mul | BinOp::Div | BinOp::Rem => 10,
    }
}

fn expect_msg(toks: &[Tok], found: &Tok) -> String {
    let mut msg = String::from("expected ");
    for (i, t) in toks.iter().enumerate() {
        if i > 0 && i != toks.len() - 1 {
            msg.push_str(", ");
        }
        if i == toks.len() - 1 && i != 0 {
            msg.push_str(" or ");
        }
        msg.push_str(&t.to_string());
    }
    msg.push_str(&format!(", but found {}", found));
    msg
}
