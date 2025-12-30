use crate::{
    ast::{
        Ast,
        expr::{BinOp, Expr, ExprId, ExprKind, UniOp},
    },
    lexer::{Lexer, Span, Tok},
};

#[derive(Debug)]
pub struct ParserError {
    pub message: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct Parser<'input> {
    lexer: Lexer<'input>,
    pub ast: Ast,
    pub errors: Vec<ParserError>,
}
impl<'input> Parser<'input> {
    pub fn new(input: &'input str) -> Self {
        Parser {
            lexer: Lexer::new(input),
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

    fn sync(&mut self) {
        loop {
            let peek = self.lexer.peek();
            if matches!(
                peek.kind,
                Tok::Semicolon | Tok::RParen | Tok::RBrace | Tok::Comma | Tok::Eof
            ) {
                break;
            }
            self.lexer.advance();
        }
    }

    fn parse_index(&mut self, base: ExprId) -> ExprId {
        let lbracket = self.lexer.advance();
        debug_assert_eq!(lbracket.kind, Tok::LBracket);

        // handle empty index
        let (index, end_span) = if self.lexer.peek().kind == Tok::RBracket {
            let rbracket = self.lexer.advance();
            let err_span = lbracket.span.between(&rbracket.span);
            self.error("expected expression inside square brackets", err_span);

            (
                self.ast.add_expr(Expr {
                    kind: ExprKind::Error,
                    span: err_span,
                }),
                rbracket.span.end,
            )
        } else {
            let index = self.parse_expr();
            let end_span = if self.lexer.peek().kind == Tok::RBracket {
                let rbracket = self.lexer.advance();
                rbracket.span.end
            } else {
                let peek = *self.lexer.peek();
                self.error(format!("expected `]`, found {}", peek.kind), peek.span);
                self.sync();

                self.ast.get_expr(index).span.end
            };
            (index, end_span)
        };

        self.ast.add_expr(Expr {
            kind: ExprKind::Index { base, index },
            span: Span {
                start: self.ast.get_expr(base).span.start,
                end: end_span,
            },
        })
    }
    fn parse_call(&mut self, callee: ExprId) -> ExprId {
        let lparen = self.lexer.advance();
        debug_assert_eq!(lparen.kind, Tok::LParen);

        let mut args = Vec::new();
        let end = loop {
            let peek = self.lexer.peek();
            if peek.kind == Tok::RParen {
                let rparen = self.lexer.advance();
                break rparen.span.end;
            }

            args.push(self.parse_expr());

            if self.lexer.peek().kind == Tok::Comma {
                self.lexer.advance();
            } else if self.lexer.peek().kind == Tok::RParen {
                let rparen = self.lexer.advance();
                break rparen.span.end;
            } else {
                let peek = *self.lexer.peek();
                self.error(
                    format!("expected `,` or `)`, found {}", peek.kind),
                    peek.span,
                );
                self.sync();
                break self.ast.get_expr(*args.last().unwrap()).span.end;
            }
        };
        self.ast.add_expr(Expr {
            kind: ExprKind::Call { callee, args },
            span: Span {
                start: self.ast.get_expr(callee).span.start,
                end,
            },
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
        // TODO: dont use recursion for unary expressions
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
                let expr = self.parse_unary_expr();
                self.ast.add_expr(Expr {
                    kind: ExprKind::UniOp { op, expr },
                    span: next.span.join(&self.ast.get_expr(expr).span),
                })
            }
            _ => self.parse_postfix_expr(),
            // _ => self.parse_term(),
        }
    }
    fn parse_binop_expr(&mut self, min_prec: u8) -> ExprId {
        let mut lhs = self.parse_unary_expr();

        loop {
            let token = self.lexer.peek();

            // check if the next token is a binary operator
            let Some(op) = bin_op_from_tok(&token.kind) else {
                break;
            };
            // check operator precedence
            if op.precedence() < min_prec {
                break;
            }
            let _op_token = self.lexer.advance();

            let rhs = self.parse_binop_expr(op.precedence() + 1);

            let start_span = self.ast.get_expr(lhs).span;
            let end_span = self.ast.get_expr(rhs).span;
            let span = start_span.join(&end_span);

            lhs = self.ast.add_expr(Expr {
                kind: ExprKind::BinOp { op, lhs, rhs },
                span,
            });
        }

        lhs
    }
    fn parse_term(&mut self) -> ExprId {
        let peek = self.lexer.peek();
        match peek.kind {
            Tok::Number(num) => {
                let number = self.lexer.advance();
                let expr = match num.parse() {
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
            Tok::Identifier(name) => {
                let ident = self.lexer.advance();
                let symbol = self.ast.intern_symbol(name);
                self.ast.add_expr(Expr {
                    kind: ExprKind::Identifier(symbol),
                    span: ident.span,
                })
            }
            Tok::LParen => {
                let lparen = self.lexer.advance();
                if self.lexer.peek().kind == Tok::RParen {
                    let rparen = self.lexer.advance();
                    let span = lparen.span.between(&rparen.span);
                    self.error("expected expression inside parenthesis", span);
                    return self.ast.add_expr(Expr {
                        kind: ExprKind::Error,
                        span,
                    });
                }

                let expr = self.parse_expr();

                let span = if self.lexer.peek().kind == Tok::RParen {
                    let rparen = self.lexer.advance();
                    lparen.span.join(&rparen.span)
                } else {
                    let peek = *self.lexer.peek();
                    // TODO: show user where the opening paren was
                    self.error(format!("expected `)`, found {}", peek.kind), peek.span);
                    // TODO: attempt to recover better like keeping track of paren depth instead
                    // of just syncing
                    self.sync();

                    lparen.span.join(&self.ast.get_expr(expr).span)
                };
                self.ast.add_expr(Expr {
                    kind: ExprKind::Group(expr),
                    span,
                })
            }
            _ => {
                let err_tok = *peek;
                self.error(
                    format!("unexpected token: {}, expected expression", err_tok.kind),
                    err_tok.span,
                );
                self.sync();

                self.ast.add_expr(Expr {
                    kind: ExprKind::Error,
                    span: err_tok.span,
                })
            }
        }
    }

    fn parse_expr(&mut self) -> ExprId {
        self.parse_binop_expr(0)
    }

    pub fn parse(&mut self) -> ExprId {
        let expr = self.parse_expr();

        let next = self.lexer.advance();
        if next.kind != Tok::Semicolon {
            self.errors.push(ParserError {
                message: format!("expected `;` or operator, found {}", next.kind),
                span: next.span,
            });
        }
        expr
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
