use std::str::Chars;
use std::vec::IntoIter;
use std::iter::Peekable;

pub struct Scanner {
    source: String,
    chars: Vec<char>,
    start: usize,
    current: usize,
    line: usize

}

#[derive(Debug,PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus,
    Semicolon, Slash, Star,
    
    // One or two character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
    
    // Literals.
    Identifier, String, Number,
    
    // Keywords.
    And, Class, Else, False,
    For, Fun, If, Nil, Or,
    Print, Return, Super, This,
    True, Var, While,
    
    Error,
    Eof,    
}

pub struct Token {
    pub tpe: TokenType,
    pub text: String,
    pub line: usize
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source: source.to_string(),
            chars: source.chars().collect::<Vec<_>>(),
            start: 0,
            current: 0,
            line: 1
        }
    }

    fn advance(&mut self) -> Option<&char> {
        let c = self.chars.get(self.current);
        self.current += 1;
        c
    }

    fn peek(&self) -> Option<&char> {
        self.chars.get(self.current)
    }

    fn peek_next(&self) -> Option<&char> {
        self.chars.get(1 + self.current)
    }

    fn matchChar(&mut self, expected: char) -> bool {
        match self.chars.get(self.current) {
            Some(c) if *c == expected => {
                self.current += 1;
                return true
            }
            _ => return false
        }
    }

    // mut to make equality in match happy ?
    pub fn scan(&mut self) -> Token {
        self.skip_whitespace();

        self.start = self.current;

        let c = self.advance();

        match c {
            None => self.make_eof(),
            Some(c) =>
                match c {
                    d if '_' == *d || d.is_alphabetic() => self.identifier(),
                    d if d.is_digit(10) => self.number(),
                    '(' => self.make_token(TokenType::LeftParen),
                    ')' => self.make_token(TokenType::RightParen),
                    '{' => self.make_token(TokenType::LeftBrace),
                    '}' => self.make_token(TokenType::RightBrace),
                    ';' => self.make_token(TokenType::Semicolon),
                    ',' => self.make_token(TokenType::Comma),
                    '.' => self.make_token(TokenType::Dot),
                    '-' => self.make_token(TokenType::Minus),
                    '+' => self.make_token(TokenType::Plus),
                    '/' => self.make_token(TokenType::Slash),
                    '*' => self.make_token(TokenType::Star),
                    '!' => {
                        let x = if self.matchChar('=') { TokenType::BangEqual } else { TokenType::Bang };
                        self.make_token(x)
                    }
                    '='=> {
                        let x =if self.matchChar('=') { TokenType::EqualEqual } else { TokenType::Equal };
                        self.make_token(x)
                    }
                    '<' => {
                        let x = if self.matchChar('=') { TokenType::LessEqual } else { TokenType::Less };
                        self.make_token(x)
                    }
                    '>' => {
                        let x = if self.matchChar('=') { TokenType::GreaterEqual } else { TokenType::Greater };
                        self.make_token(x)
                    }
                    '"' => self.string(),
                    _   => self.error_token("Unexpected character.")
                }
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(' ') | Some('\r') | Some('\t') => {
                    self.advance();
                }
                Some ('\n') => {
                    self.line += 1;
                    self.advance();
                }
                Some ('/') => { 
                    match self.peek_next() {
                        Some('/') => {
                            loop {
                                match self.peek() {
                                    Some('\n') | None => { break }
                                    _ => { self.advance(); }
                                }
                            }
                        } 
                        _ => {}
                }}
                _ => {
                    return
                }
            }
        }
    }

    fn check_keyword(&self, start: usize, length: usize, rest: &str, tpe: TokenType) -> TokenType {
        if self.current - self.start == start + length &&
            String::from(& self.source[self.start + start .. self.start + start + length]) == String::from(rest) {
            tpe
        } else {
            TokenType::Identifier
        }
    }

    fn identifier_type(&self) -> TokenType {
        match self.chars[self.start] {
            'a' => self.check_keyword(1, 2, "nd", TokenType::And),
            'c' => self.check_keyword(1, 4, "lass", TokenType::Class),
            'e' => self.check_keyword(1, 3, "lse", TokenType::Else),
            'i' => self.check_keyword(1, 1, "f", TokenType::If),
            'f' if self.current - self.start > 1 => {
                match self.chars[self.start + 1] {
                        'a'=> return self.check_keyword(2, 3, "lse", TokenType::False),
                        'o'=> return self.check_keyword(2, 1, "r", TokenType::For),
                        'u'=> return self.check_keyword(2, 1, "n", TokenType::Fun),
                        _ => TokenType::Identifier
                    }
                }
            'n' => self.check_keyword(1, 2, "il", TokenType::Nil),
            'o' => self.check_keyword(1, 1, "r", TokenType::Or),
            'p' => self.check_keyword(1, 4, "rint", TokenType::Print),
            'r' => self.check_keyword(1, 5, "eturn", TokenType::Return),
            's' => self.check_keyword(1, 4, "uper", TokenType::Super),
            't' if self.current - self.start > 1 => {
                match self.chars[self.start + 1] {
                        'h'=> return self.check_keyword(2, 2, "is", TokenType::This),
                        'r'=> return self.check_keyword(2, 2, "ue", TokenType::True),
                        _ => TokenType::Identifier
                    }
                }
            'v' => self.check_keyword(1, 2, "ar", TokenType::Var),
            'w' => self.check_keyword(1, 4, "hile", TokenType::While),
            _ => TokenType::Identifier,
          }
    }

    fn identifier(&mut self) -> Token {
        loop {
            match self.peek() {
                Some(c) if c.is_digit(10) || c.is_alphabetic() || *c == '_' => {
                    self.advance();
                }
                _ => return self.make_token(self.identifier_type())
            }
        }
    }

    fn number_fragment(&mut self) {
        loop {
            match self.peek() {
                Some(d) if d.is_digit(10) => { 
                    self.advance(); 
                }
                _ => break  
            }
        }
    }
    fn number(&mut self) -> Token {
        self.number_fragment();
        // Look for a fractional part.
        match (self.peek(), self.peek_next()) {
            (Some('.'), Some(d)) if d.is_digit(10) => {
                // Consume the ".".
                self.advance();

                self.number_fragment();
            }
            _ => {}
        }      
      
        return self.make_token(TokenType::Number);
      }

    fn string(&mut self) -> Token {
        loop {
            match self.peek() {
                None => return self.error_token("Unterminated string."),
                Some('\n') => { self.line += 1; }
                Some('"') => {
                    self.advance();
                    break;
                }
                _ => { self.advance(); }
            }
        }
        self.advance();
        return self.make_token(TokenType::String);
    }

    fn make_eof(&self) -> Token {
        Token {
            tpe:     TokenType::Eof,
            text: String::from(&self.source[self.start..self.current-1]),
            line: self.line
        }
    }

    fn make_token(&self, tpe: TokenType) -> Token {
        Token {
            tpe: tpe,
            text: String::from(&self.source[self.start..self.current]),
            line: self.line
        }
    }

    fn error_token(&self, message: &str) -> Token {
        Token {
            tpe: TokenType::Error,
            text: String::from(message),
            line: self.line
        }
      }
}
