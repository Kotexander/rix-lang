use std::str::Chars;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tok {
    Number,
    String,
    Identifier,

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

    Unknown,

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
impl std::fmt::Display for Tok {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tok::Number => write!(f, "number literal"),
            Tok::String => write!(f, "string literal"),
            Tok::Identifier => write!(f, "identifier"),
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
            Tok::Fun => write!(f, "`fun`"),
            Tok::Return => write!(f, "`return`"),
            Tok::Var => write!(f, "`var`"),
            Tok::If => write!(f, "`if`"),
            Tok::Else => write!(f, "`else`"),
            Tok::While => write!(f, "`while`"),
            Tok::Break => write!(f, "`break`"),
            Tok::Continue => write!(f, "`continue`"),
            Tok::Unknown => write!(f, "unknown character"),
            Tok::Eof => write!(f, "<EOF>"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

    pub fn len(&self) -> u32 {
        self.end - self.start
    }
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}
impl std::ops::Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.start as usize..index.end as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: Tok,
    pub span: Span,
}

#[derive(Clone)]
pub struct Lexer<'input> {
    input: &'input str,
    chars: Chars<'input>,
}
impl std::fmt::Debug for Lexer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lexer").finish_non_exhaustive()
    }
}
impl<'input> Lexer<'input> {
    /// Create a new lexer for the given input string.
    /// # Panics
    /// Panics if the input string is larger than `u32::MAX` bytes.
    pub fn new(input: &'input str) -> Self {
        assert!(input.len() < u32::MAX as usize, "input is too large");

        Lexer {
            input,
            chars: input.chars(),
        }
    }

    pub fn slice(&self, span: Span) -> &'input str {
        &self.input[span]
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

    pub fn advance(&mut self) -> Token {
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
                    Tok::Number
                }
                '"' => {
                    self.eat_string();
                    Tok::String
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
                ch if ch == '_' || ch.is_ascii_alphabetic() => {
                    self.eat_word();
                    let end = self.location();
                    let word = &self.input[start as usize..end as usize];
                    match word {
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
                        _ => Tok::Identifier,
                    }
                }
                _ => Tok::Unknown,
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
        self.eat_word();
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

    fn eat_word(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch == '_' || ch.is_ascii_alphanumeric() {
                self.next_char();
            } else {
                break;
            }
        }
    }
}

/// A [Lexer] wrapper that provides a sliding window of previous, and next tokens.
#[derive(Debug, Clone)]
pub struct LexerWindow<'input> {
    lexer: Lexer<'input>,
    peek: Option<Token>,
    prev: Option<Token>,
}
impl<'input> LexerWindow<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            lexer: Lexer::new(input),
            peek: None,
            prev: None,
        }
    }

    pub fn slice(&self, span: Span) -> &'input str {
        self.lexer.slice(span)
    }

    pub fn peek(&mut self) -> &Token {
        self.peek.get_or_insert_with(|| self.lexer.advance())
    }
    pub fn prev(&self) -> &Token {
        self.prev
            .as_ref()
            .expect("prev() should not be called on the first token")
    }

    pub fn advance(&mut self) -> Token {
        let curr = self.peek.take().unwrap_or_else(|| self.lexer.advance());
        self.prev = Some(curr);
        curr
    }

    /// Attempt to match the next token with the given token kind.
    /// - If it matches, advance the lexer and return `Ok` with the matched token.
    /// - If it does not match, returns `Err` with the current peek token.
    pub fn match_(&mut self, tok: Tok) -> Result<Span, Token> {
        if self.peek().kind == tok {
            Ok(self.advance().span)
        } else {
            Err(*self.peek())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers() {
        let mut lexer = Lexer::new("123 456 7eight_nine");

        let tok = lexer.advance();
        assert_eq!(tok.kind, Tok::Number);
        assert_eq!(&lexer.input[tok.span], "123");

        let tok = lexer.advance();
        assert_eq!(tok.kind, Tok::Number);
        assert_eq!(&lexer.input[tok.span], "456");

        let tok = lexer.advance();
        assert_eq!(tok.kind, Tok::Number);
        assert_eq!(&lexer.input[tok.span], "7eight_nine");

        assert_eq!(lexer.advance().kind, Tok::Eof);
    }

    #[test]
    fn strings() {
        let mut lexer = Lexer::new(r#" "hello" "var x = \"Hello!\"" "#);

        let tok = lexer.advance();
        assert_eq!(tok.kind, Tok::String);
        assert_eq!(&lexer.input[tok.span], r#""hello""#);
        let tok = lexer.advance();
        assert_eq!(tok.kind, Tok::String);
        assert_eq!(&lexer.input[tok.span], r#""var x = \"Hello!\"""#);

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
    fn example() {
        let mut lexer = Lexer::new(
            r#"
            # This is a comment
            var result = add(5, 10);
            # Another comment
            fun add(a, b) : i32 {
                return a + b;
            }
            "#,
        );
        assert_eq!(lexer.advance().kind, Tok::Var);
        assert_eq!(lexer.advance().kind, Tok::Identifier);
        assert_eq!(lexer.advance().kind, Tok::Equal);
        assert_eq!(lexer.advance().kind, Tok::Identifier);
        assert_eq!(lexer.advance().kind, Tok::LParen);
        assert_eq!(lexer.advance().kind, Tok::Number);
        assert_eq!(lexer.advance().kind, Tok::Comma);
        assert_eq!(lexer.advance().kind, Tok::Number);
        assert_eq!(lexer.advance().kind, Tok::RParen);
        assert_eq!(lexer.advance().kind, Tok::Semicolon);
        assert_eq!(lexer.advance().kind, Tok::Fun);
        assert_eq!(lexer.advance().kind, Tok::Identifier);
        assert_eq!(lexer.advance().kind, Tok::LParen);
        assert_eq!(lexer.advance().kind, Tok::Identifier);
        assert_eq!(lexer.advance().kind, Tok::Comma);
        assert_eq!(lexer.advance().kind, Tok::Identifier);
        assert_eq!(lexer.advance().kind, Tok::RParen);
        assert_eq!(lexer.advance().kind, Tok::Colon);
        assert_eq!(lexer.advance().kind, Tok::Identifier);
        assert_eq!(lexer.advance().kind, Tok::LBrace);
        assert_eq!(lexer.advance().kind, Tok::Return);
        assert_eq!(lexer.advance().kind, Tok::Identifier);
        assert_eq!(lexer.advance().kind, Tok::Plus);
        assert_eq!(lexer.advance().kind, Tok::Identifier);
        assert_eq!(lexer.advance().kind, Tok::Semicolon);
        assert_eq!(lexer.advance().kind, Tok::RBrace);
        assert_eq!(lexer.advance().kind, Tok::Eof);
    }

    #[test]
    fn window() {
        let input = "var x = a + b";
        let mut lexer = LexerWindow::new(input);

        assert_eq!(lexer.advance().kind, Tok::Var);
        assert_eq!(lexer.advance().kind, Tok::Identifier);

        assert_eq!(lexer.prev().kind, Tok::Identifier);
        assert_eq!(lexer.peek().kind, Tok::Equal);
        assert_eq!(lexer.advance().kind, Tok::Equal);

        assert_eq!(lexer.peek().kind, Tok::Identifier);
        assert_eq!(lexer.prev().kind, Tok::Equal);
        assert_eq!(lexer.advance().kind, Tok::Identifier);

        assert_eq!(lexer.prev().kind, Tok::Identifier);
        assert_eq!(lexer.peek().kind, Tok::Plus);
        assert_eq!(lexer.advance().kind, Tok::Plus);

        assert_eq!(lexer.advance().kind, Tok::Identifier);
        assert_eq!(lexer.advance().kind, Tok::Eof);
    }
}
