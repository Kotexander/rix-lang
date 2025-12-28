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
    Newline,

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'a> {
    pub kind: Tok<'a>,
    pub start: u32,
    pub end: u32,
}

pub struct Lexer<'input> {
    input: &'input str,
    chars: Chars<'input>,
}
impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        assert!(input.len() < u32::MAX as usize, "input is too large");
        Lexer {
            input,
            chars: input.chars(),
        }
    }

    fn location(&self) -> u32 {
        (self.input.len() - self.chars.as_str().len()) as u32
    }
    fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }
    fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn r#match(&mut self, ch: char) -> bool {
        if let Some(next_char) = self.peek() {
            if next_char == ch {
                self.next();
                return true;
            }
        }
        false
    }

    pub fn advance(&mut self) -> Token<'input> {
        loop {
            let start = self.location();
            let Some(ch) = self.next() else {
                return Token {
                    kind: Tok::Eof,
                    start,
                    end: start,
                };
            };

            let tok = match ch {
                '\n' => Tok::Newline,
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
                    if self.r#match('=') {
                        Tok::BangEqual
                    } else {
                        Tok::Bang
                    }
                }
                '&' => Tok::Ampersand,
                '^' => Tok::Caret,
                '|' => Tok::Pipe,
                '=' => {
                    if self.r#match('=') {
                        Tok::EqualEqual
                    } else {
                        Tok::Equal
                    }
                }
                '<' => {
                    if self.r#match('=') {
                        Tok::LessEqual
                    } else if self.r#match('<') {
                        Tok::LessLess
                    } else {
                        Tok::Less
                    }
                }
                '>' => {
                    if self.r#match('=') {
                        Tok::GreaterEqual
                    } else if self.r#match('>') {
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
                    if self.r#match('.') {
                        if self.r#match('.') {
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
                        _ => Tok::Identifier(identifier),
                    }
                }
                _ => Tok::Unknown(ch),
            };

            let end = self.location();
            return Token {
                kind: tok,
                start,
                end,
            };
        }
    }

    fn eat_numbers(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.next();
            } else {
                break;
            }
        }
    }

    fn eat_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    fn eat_string(&mut self) {
        let mut escaped = false;
        while let Some(ch) = self.next() {
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
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            } else {
                self.next();
            }
        }
    }

    fn eat_identifier(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.next();
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
        let mut lexer = Lexer::new("123 456");
        assert_eq!(lexer.advance().kind, Tok::Number("123"));
        assert_eq!(lexer.advance().kind, Tok::Number("456"));
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
        assert_eq!(token.start, 0);
        assert_eq!(token.end, 3);

        let token = lexer.advance();
        assert_eq!(token.kind, Tok::Identifier("x"));
        assert_eq!(token.start, 4);
        assert_eq!(token.end, 5);

        let token = lexer.advance();
        assert_eq!(token.kind, Tok::Equal);
        assert_eq!(token.start, 6);
        assert_eq!(token.end, 7);

        let token = lexer.advance();
        assert_eq!(token.kind, Tok::Number("42"));
        assert_eq!(token.start, 8);
        assert_eq!(token.end, 10);
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
        assert_eq!(lexer.advance().kind, Tok::Newline);
        assert_eq!(lexer.advance().kind, Tok::Newline);
        assert_eq!(lexer.advance().kind, Tok::Var);
        assert_eq!(lexer.advance().kind, Tok::Identifier("result"));
        assert_eq!(lexer.advance().kind, Tok::Equal);
        assert_eq!(lexer.advance().kind, Tok::Identifier("add"));
        assert_eq!(lexer.advance().kind, Tok::LParen);
        assert_eq!(lexer.advance().kind, Tok::Number("5"));
        assert_eq!(lexer.advance().kind, Tok::Comma);
        assert_eq!(lexer.advance().kind, Tok::Number("10"));
        assert_eq!(lexer.advance().kind, Tok::RParen);
        assert_eq!(lexer.advance().kind, Tok::Newline);
        assert_eq!(lexer.advance().kind, Tok::Newline);
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
        assert_eq!(lexer.advance().kind, Tok::Newline);
        assert_eq!(lexer.advance().kind, Tok::Return);
        assert_eq!(lexer.advance().kind, Tok::Identifier("a"));
        assert_eq!(lexer.advance().kind, Tok::Plus);
        assert_eq!(lexer.advance().kind, Tok::Identifier("b"));
        assert_eq!(lexer.advance().kind, Tok::Newline);
        assert_eq!(lexer.advance().kind, Tok::RBrace);
        assert_eq!(lexer.advance().kind, Tok::Newline);
        assert_eq!(lexer.advance().kind, Tok::Eof);
    }
}
