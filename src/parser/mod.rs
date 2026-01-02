use crate::{
    ast::{
        Ast,
        expr::{BinOp, Expr, ExprId, ExprKind, UniOp},
        stmt::Stmt,
    },
    lexer::{LexerWindow, Span, Tok, Token},
};

#[derive(Debug)]
pub struct ParserError {
    pub message: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct Parser<'input> {
    lexer: LexerWindow<'input>,
    pub ast: Ast,
    pub errors: Vec<ParserError>,
}
impl<'input> Parser<'input> {
    pub fn new(input: &'input str) -> Self {
        Parser {
            lexer: LexerWindow::new(input),
            ast: Ast::new(),
            errors: Vec::new(),
        }
    }

    fn error(&mut self, message: impl Into<String>, span: Span) {
        self.errors.push(ParserError {
            message: message.into(),
            span,
        });
    }

    fn expect(&mut self, tok: &[Tok]) -> Token {
        let peek = *self.lexer.peek();
        if tok.contains(&peek.kind) {
            return self.lexer.advance();
        }
        let msg = expect_msg(tok, &peek.kind);
        self.error(msg, peek.span);
        peek
    }

    fn parse_group(&mut self) -> ExprId {
        let lparen = self.lexer.advance();
        debug_assert_eq!(lparen.kind, Tok::LParen);

        if let Ok(rparen) = self.lexer.match_(Tok::RParen) {
            self.error(
                "expected expression inside parenthesis",
                lparen.span.between(&rparen.span),
            );
            return self.ast.add_expr(Expr {
                kind: ExprKind::Error,
                span: lparen.span.join(&rparen.span),
            });
        }

        let expr = self.parse_expr();

        self.expect(&[Tok::RParen]);

        self.ast.add_expr(Expr {
            kind: ExprKind::Group(expr),
            span: lparen.span.join(&self.lexer.prev().span),
        })
    }
    fn parse_term(&mut self) -> ExprId {
        let peek = self.lexer.peek();
        match peek.kind {
            Tok::Number => {
                let number = self.lexer.advance();
                let expr = match self.lexer.slice(number.span).parse() {
                    Ok(n) => ExprKind::Integer(n),
                    Err(e) => {
                        self.error(format!("invalid integer literal: {}", e), number.span);
                        ExprKind::Error
                    }
                };
                self.ast.add_expr(Expr {
                    kind: expr,
                    span: number.span,
                })
            }
            Tok::Identifier => {
                let identifer = self.lexer.advance();
                let symbol = self.ast.get_symbol(self.lexer.slice(identifer.span));
                self.ast.add_expr(Expr {
                    kind: ExprKind::Identifier(symbol),
                    span: identifer.span,
                })
            }
            Tok::LParen => self.parse_group(),
            _ => {
                let token = self.lexer.advance();
                self.error(
                    format!("expected expression, but found {}", token.kind),
                    token.span,
                );

                self.ast.add_expr(Expr {
                    kind: ExprKind::Error,
                    span: token.span,
                })
            }
        }
    }

    fn parse_index(&mut self, base: ExprId) -> ExprId {
        let lbracket = self.lexer.advance();
        debug_assert_eq!(lbracket.kind, Tok::LBracket);

        // handle empty index
        let index = if let Ok(rbracket) = self.lexer.match_(Tok::RBracket) {
            let err_span = lbracket.span.between(&rbracket.span);
            self.error("expected expression for index operation", err_span);

            self.ast.add_expr(Expr {
                kind: ExprKind::Error,
                span: err_span,
            })
        } else {
            let index = self.parse_expr();
            self.expect(&[Tok::RBracket]);
            index
        };

        self.ast.add_expr(Expr {
            kind: ExprKind::Index { base, index },
            span: self.ast.get_expr(base).span.join(&self.lexer.prev().span),
        })
    }
    fn parse_call(&mut self, callee: ExprId) -> ExprId {
        let lparen = self.lexer.advance();
        debug_assert_eq!(lparen.kind, Tok::LParen);

        let mut args = Vec::new();
        loop {
            if self.lexer.match_(Tok::RParen).is_ok() {
                break;
            }

            args.push(self.parse_expr());

            match self.lexer.peek().kind {
                Tok::Comma => {
                    self.lexer.advance();
                }
                Tok::RParen => {
                    let _rparen = self.lexer.advance();
                    break;
                }
                _ => {
                    let peek = *self.lexer.peek();
                    self.error(
                        expect_msg(&[Tok::Comma, Tok::RParen], &peek.kind),
                        peek.span,
                    );
                    break;
                }
            }
        }
        self.ast.add_expr(Expr {
            kind: ExprKind::Call { callee, args },
            span: self.ast.get_expr(callee).span.join(&self.lexer.prev().span),
        })
    }
    fn parse_postfix_expr(&mut self) -> ExprId {
        let mut expr = self.parse_term();

        loop {
            let peek = self.lexer.peek();
            match peek.kind {
                Tok::LBracket => {
                    expr = self.parse_index(expr);
                }
                Tok::LParen => {
                    expr = self.parse_call(expr);
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_unary_expr(&mut self) -> ExprId {
        // TODO: use small vec instead of allocating right away
        let mut ops = vec![];
        loop {
            let peek = self.lexer.peek();
            match peek.kind {
                Tok::Minus | Tok::Bang | Tok::Ampersand | Tok::Asterisk => {
                    let next = self.lexer.advance();
                    let op = match next.kind {
                        Tok::Minus => UniOp::Neg,
                        Tok::Bang => UniOp::Not,
                        Tok::Ampersand => UniOp::Ref,
                        Tok::Asterisk => UniOp::Deref,
                        _ => unreachable!(),
                    };
                    ops.push((op, next.span));
                }
                _ => break,
            }
        }

        let mut expr = self.parse_postfix_expr();
        for (op, op_span) in ops.into_iter().rev() {
            let expr_span = self.ast.get_expr(expr).span;
            let span = op_span.join(&expr_span);
            expr = self.ast.add_expr(Expr {
                kind: ExprKind::UniOp { op, expr },
                span,
            });
        }

        expr
    }

    fn parse_binop_expr(&mut self, min_prec: u8) -> ExprId {
        let mut lhs = self.parse_unary_expr();

        loop {
            let peek = self.lexer.peek();

            // check if the next token is a binary operator
            let Some(op) = bin_op_from_tok(&peek.kind) else {
                break;
            };
            // check operator precedence
            if op.precedence() < min_prec {
                break;
            }
            let _op_token = self.lexer.advance();

            let rhs = self.parse_binop_expr(op.precedence() + 1);

            let lhs_span = self.ast.get_expr(lhs).span;
            let rhs_span = self.ast.get_expr(rhs).span;
            let span = lhs_span.join(&rhs_span);

            lhs = self.ast.add_expr(Expr {
                kind: ExprKind::BinOp { op, lhs, rhs },
                span,
            });
        }

        lhs
    }

    fn parse_expr(&mut self) -> ExprId {
        self.parse_binop_expr(0)
    }

    pub fn parse(&mut self) {
        loop {
            let peek = self.lexer.peek();
            let stmt = match peek.kind {
                Tok::Var => {
                    let _var = self.lexer.advance();
                    let name = self.expect(&[Tok::Identifier]);
                    self.expect(&[Tok::Equal]);
                    let expr = self.parse_expr();
                    self.expect(&[Tok::Semicolon]);
                    Stmt::VarDecl {
                        name: self.ast.get_symbol(self.lexer.slice(name.span)),
                        name_span: name.span,
                        value: expr,
                    }
                }
                Tok::Eof => break,
                _ => {
                    let expr = self.parse_expr();
                    self.expect(&[Tok::Semicolon]);
                    Stmt::Expr(expr)
                }
            };
            self.ast.add_stmt(stmt);
        }
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
