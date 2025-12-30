use std::str::Chars;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tok<'a> {
    Number(&'a str),
    String(&'a str),
    Identifier(&'a str),

    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Bang,
    Ampersand,
    Caret,
    Pipe,
    And,
    Or,

    Equal,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    LessLess,
    Greater,
    GreaterEqual,
    GreaterGreater,

    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    Dot,
    DotDot,
    DotDotDot,
    Comma,
    Colon,
    Semicolon,

    Unknown(char),

    Fun,
    Return,
    Var,
    If,
    Else,
    While,
    Break,
    Continue,

    Eof,
}
impl<'a> Tok<'a> {
    pub fn is_eof(self) -> bool {
        self == Tok::Eof
    }
}
impl<'a> std::fmt::Display for Tok<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tok::Number(n) => write!(f, "number `{:?}`", n),
            Tok::String(s) => write!(f, "string `{:?}`", s),
            Tok::Identifier(id) => write!(f, "identifier `{}`", id),
            Tok::Plus => write!(f, "`+`"),
            Tok::Minus => write!(f, "`-`"),
            Tok::Asterisk => write!(f, "`*`"),
            Tok::Slash => write!(f, "`/`"),
            Tok::Percent => write!(f, "`%`"),
            Tok::Bang => write!(f, "`!`"),
            Tok::Ampersand => write!(f, "`&`"),
            Tok::Caret => write!(f, "`^`"),
            Tok::Pipe => write!(f, "`|`"),
            Tok::And => write!(f, "`and`"),
            Tok::Or => write!(f, "`or`"),
            Tok::Equal => write!(f, "`=`"),
            Tok::EqualEqual => write!(f, "`==`"),
            Tok::BangEqual => write!(f, "`!=`"),
            Tok::Less => write!(f, "`<`"),
            Tok::LessEqual => write!(f, "`<=`"),
            Tok::LessLess => write!(f, "`<<`"),
            Tok::Greater => write!(f, "`>`"),
            Tok::GreaterEqual => write!(f, "`>=`"),
            Tok::GreaterGreater => write!(f, "`>>`"),
            Tok::LParen => write!(f, "`(`"),
            Tok::RParen => write!(f, "`)`"),
            Tok::LBrace => write!(f, "`{{`"),
            Tok::RBrace => write!(f, "`}}`"),
            Tok::LBracket => write!(f, "`[`"),
            Tok::RBracket => write!(f, "`]`"),
            Tok::Dot => write!(f, "`.`"),
            Tok::DotDot => write!(f, "`..`"),
            Tok::DotDotDot => write!(f, "`...`"),
            Tok::Comma => write!(f, "`,`"),
            Tok::Colon => write!(f, "`:`"),
            Tok::Semicolon => write!(f, "`;`"),
            Tok::Unknown(ch) => write!(f, "`{}`", ch),
            Tok::Fun => write!(f, "`fun`"),
            Tok::Return => write!(f, "`return`"),
            Tok::Var => write!(f, "`var`"),
            Tok::If => write!(f, "`if`"),
            Tok::Else => write!(f, "`else`"),
            Tok::While => write!(f, "`while`"),
            Tok::Break => write!(f, "`break`"),
            Tok::Continue => write!(f, "`continue`"),
            Tok::Eof => write!(f, "<EOF>"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}
impl Span {
    pub fn join(&self, other: &Span) -> Span {
        Span {
            start: self.start,
            end: other.end,
        }
    }
    pub fn between(&self, other: &Span) -> Span {
        Span {
            start: self.end,
            end: other.start,
        }
    }
}
impl std::ops::Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.start as usize..index.end as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'a> {
    pub kind: Tok<'a>,
    pub span: Span,
}

pub struct Lexer<'input> {
    input: &'input str,
    chars: Chars<'input>,

    peek: Option<Token<'input>>,
}
impl std::fmt::Debug for Lexer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lexer").finish_non_exhaustive()
    }
}
impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        assert!(input.len() < u32::MAX as usize, "input is too large");
        Lexer {
            input,
            chars: input.chars(),
            peek: None,
        }
    }

    fn location(&self) -> u32 {
        (self.input.len() - self.chars.as_str().len()) as u32
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.clone().next()
    }
    fn next_char(&mut self) -> Option<char> {
        self.chars.next()
    }
    fn match_char(&mut self, ch: char) -> bool {
        if let Some(next_char) = self.peek_char()
            && next_char == ch
        {
            self.next_char();
            return true;
        }
        false
    }

    pub fn peek(&mut self) -> &Token<'input> {
        if self.peek.is_none() {
            let token = self.advance();
            self.peek = Some(token);
        }
        // Safety: We just ensured that peek is Some.
        unsafe { self.peek.as_ref().unwrap_unchecked() }
    }
    pub fn advance(&mut self) -> Token<'input> {
        if let Some(peek) = self.peek.take() {
            return peek;
        }
        loop {
            let start = self.location();
            let Some(ch) = self.next_char() else {
                return Token {
                    kind: Tok::Eof,
                    span: Span { start, end: start },
                };
            };

            let tok = match ch {
                ch if ch.is_whitespace() => {
                    self.eat_whitespace();
                    continue;
                }
                ch if ch.is_ascii_digit() => {
                    self.eat_numbers();
                    let end = self.location();
                    let number = &self.input[start as usize..end as usize];
                    Tok::Number(number)
                }
                '"' => {
                    self.eat_string();
                    let end = self.location();
                    let string = &self.input[start as usize..end as usize];
                    Tok::String(string)
                }
                '+' => Tok::Plus,
                '-' => Tok::Minus,
                '*' => Tok::Asterisk,
                '/' => Tok::Slash,
                '%' => Tok::Percent,
                '!' => {
                    if self.match_char('=') {
                        Tok::BangEqual
                    } else {
                        Tok::Bang
                    }
                }
                '&' => Tok::Ampersand,
                '^' => Tok::Caret,
                '|' => Tok::Pipe,
                '=' => {
                    if self.match_char('=') {
                        Tok::EqualEqual
                    } else {
                        Tok::Equal
                    }
                }
                '<' => {
                    if self.match_char('=') {
                        Tok::LessEqual
                    } else if self.match_char('<') {
                        Tok::LessLess
                    } else {
                        Tok::Less
                    }
                }
                '>' => {
                    if self.match_char('=') {
                        Tok::GreaterEqual
                    } else if self.match_char('>') {
                        Tok::GreaterGreater
                    } else {
                        Tok::Greater
                    }
                }
                '(' => Tok::LParen,
                ')' => Tok::RParen,
                '{' => Tok::LBrace,
                '}' => Tok::RBrace,
                '[' => Tok::LBracket,
                ']' => Tok::RBracket,
                '.' => {
                    if self.match_char('.') {
                        if self.match_char('.') {
                            Tok::DotDotDot
                        } else {
                            Tok::DotDot
                        }
                    } else {
                        Tok::Dot
                    }
                }
                ',' => Tok::Comma,
                ':' => Tok::Colon,
                ';' => Tok::Semicolon,
                '#' => {
                    self.eat_comment();
                    continue;
                }
                ch if ch.is_ascii_alphabetic() || ch == '_' => {
                    self.eat_identifier();
                    let end = self.location();
                    let identifier = &self.input[start as usize..end as usize];
                    match identifier {
                        "fun" => Tok::Fun,
                        "return" => Tok::Return,
                        "var" => Tok::Var,
                        "if" => Tok::If,
                        "else" => Tok::Else,
                        "while" => Tok::While,
                        "break" => Tok::Break,
                        "continue" => Tok::Continue,
                        "and" => Tok::And,
                        "or" => Tok::Or,
                        _ => Tok::Identifier(identifier),
                    }
                }
                _ => Tok::Unknown(ch),
            };

            let end = self.location();
            return Token {
                kind: tok,
                span: Span { start, end },
            };
        }
    }

    #[inline]
    fn eat_numbers(&mut self) {
        self.eat_identifier();
    }

    fn eat_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn eat_string(&mut self) {
        let mut escaped = false;
        while let Some(ch) = self.next_char() {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                break;
            }
        }
    }

    fn eat_comment(&mut self) {
        while let Some(ch) = self.next_char() {
            if ch == '\n' {
                break;
            }
        }
    }

    fn eat_identifier(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.next_char();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers() {
        let mut lexer = Lexer::new("123 456 7eight_nine");
        assert_eq!(lexer.advance().kind, Tok::Number("123"));
        assert_eq!(lexer.advance().kind, Tok::Number("456"));
        assert_eq!(lexer.advance().kind, Tok::Number("7eight_nine"));
        assert_eq!(lexer.advance().kind, Tok::Eof);
    }

    #[test]
    fn strings() {
        let mut lexer = Lexer::new(r#" "hello" "var x = \"Hello!\"" "#);
        assert_eq!(lexer.advance().kind, Tok::String(r#""hello""#));
        assert_eq!(lexer.advance().kind, Tok::String(r#""var x = \"Hello!\"""#));
        assert_eq!(lexer.advance().kind, Tok::Eof);
    }

    #[test]
    fn dots() {
        let mut lexer = Lexer::new(". .. ...");
        assert_eq!(lexer.advance().kind, Tok::Dot);
        assert_eq!(lexer.advance().kind, Tok::DotDot);
        assert_eq!(lexer.advance().kind, Tok::DotDotDot);
        assert_eq!(lexer.advance().kind, Tok::Eof);
    }

    #[test]
    fn long_operators() {
        let mut lexer = Lexer::new("= == != < <= << > >= >>");
        assert_eq!(lexer.advance().kind, Tok::Equal);
        assert_eq!(lexer.advance().kind, Tok::EqualEqual);
        assert_eq!(lexer.advance().kind, Tok::BangEqual);
        assert_eq!(lexer.advance().kind, Tok::Less);
        assert_eq!(lexer.advance().kind, Tok::LessEqual);
        assert_eq!(lexer.advance().kind, Tok::LessLess);
        assert_eq!(lexer.advance().kind, Tok::Greater);
        assert_eq!(lexer.advance().kind, Tok::GreaterEqual);
        assert_eq!(lexer.advance().kind, Tok::GreaterGreater);
        assert_eq!(lexer.advance().kind, Tok::Eof);
    }

    #[test]
    fn spans() {
        let mut lexer = Lexer::new("var x = 42");
        let token = lexer.advance();
        assert_eq!(token.kind, Tok::Var);
        assert_eq!(token.span.start, 0);
        assert_eq!(token.span.end, 3);

        let token = lexer.advance();
        assert_eq!(token.kind, Tok::Identifier("x"));
        assert_eq!(token.span.start, 4);
        assert_eq!(token.span.end, 5);

        let token = lexer.advance();
        assert_eq!(token.kind, Tok::Equal);
        assert_eq!(token.span.start, 6);
        assert_eq!(token.span.end, 7);

        let token = lexer.advance();
        assert_eq!(token.kind, Tok::Number("42"));
        assert_eq!(token.span.start, 8);
        assert_eq!(token.span.end, 10);
    }

    #[test]
    fn example() {
        let mut lexer = Lexer::new(
            r#"
            # This is a comment
            var result = add(5, 10)
            # Another comment
            fun add(a, b) : i32 {
                return a + b
            }
            "#,
        );
        assert_eq!(lexer.advance().kind, Tok::Var);
        assert_eq!(lexer.advance().kind, Tok::Identifier("result"));
        assert_eq!(lexer.advance().kind, Tok::Equal);
        assert_eq!(lexer.advance().kind, Tok::Identifier("add"));
        assert_eq!(lexer.advance().kind, Tok::LParen);
        assert_eq!(lexer.advance().kind, Tok::Number("5"));
        assert_eq!(lexer.advance().kind, Tok::Comma);
        assert_eq!(lexer.advance().kind, Tok::Number("10"));
        assert_eq!(lexer.advance().kind, Tok::RParen);
        assert_eq!(lexer.advance().kind, Tok::Fun);
        assert_eq!(lexer.advance().kind, Tok::Identifier("add"));
        assert_eq!(lexer.advance().kind, Tok::LParen);
        assert_eq!(lexer.advance().kind, Tok::Identifier("a"));
        assert_eq!(lexer.advance().kind, Tok::Comma);
        assert_eq!(lexer.advance().kind, Tok::Identifier("b"));
        assert_eq!(lexer.advance().kind, Tok::RParen);
        assert_eq!(lexer.advance().kind, Tok::Colon);
        assert_eq!(lexer.advance().kind, Tok::Identifier("i32"));
        assert_eq!(lexer.advance().kind, Tok::LBrace);
        assert_eq!(lexer.advance().kind, Tok::Return);
        assert_eq!(lexer.advance().kind, Tok::Identifier("a"));
        assert_eq!(lexer.advance().kind, Tok::Plus);
        assert_eq!(lexer.advance().kind, Tok::Identifier("b"));
        assert_eq!(lexer.advance().kind, Tok::RBrace);
        assert_eq!(lexer.advance().kind, Tok::Eof);
    }
}
