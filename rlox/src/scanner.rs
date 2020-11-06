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
    TOKEN_LEFT_PAREN, TOKEN_RIGHT_PAREN,
    TOKEN_LEFT_BRACE, TOKEN_RIGHT_BRACE,
    TOKEN_COMMA, TOKEN_DOT, TOKEN_MINUS, TOKEN_PLUS,
    TOKEN_SEMICOLON, TOKEN_SLASH, TOKEN_STAR,
    
    // One or two character tokens.
    TOKEN_BANG, TOKEN_BANG_EQUAL,
    TOKEN_EQUAL, TOKEN_EQUAL_EQUAL,
    TOKEN_GREATER, TOKEN_GREATER_EQUAL,
    TOKEN_LESS, TOKEN_LESS_EQUAL,
    
    // Literals.
    TOKEN_IDENTIFIER, TOKEN_STRING, TOKEN_NUMBER,
    
    // Keywords.
    TOKEN_AND, TOKEN_CLASS, TOKEN_ELSE, TOKEN_FALSE,
    TOKEN_FOR, TOKEN_FUN, TOKEN_IF, TOKEN_NIL, TOKEN_OR,
    TOKEN_PRINT, TOKEN_RETURN, TOKEN_SUPER, TOKEN_THIS,
    TOKEN_TRUE, TOKEN_VAR, TOKEN_WHILE,
    
    TOKEN_ERROR,
    TOKEN_EOF
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
                    '(' => self.make_token(TokenType::TOKEN_LEFT_PAREN),
                    ')' => self.make_token(TokenType::TOKEN_RIGHT_PAREN),
                    '{' => self.make_token(TokenType::TOKEN_LEFT_BRACE),
                    '}' => self.make_token(TokenType::TOKEN_RIGHT_BRACE),
                    ';' => self.make_token(TokenType::TOKEN_SEMICOLON),
                    ',' => self.make_token(TokenType::TOKEN_COMMA),
                    '.' => self.make_token(TokenType::TOKEN_DOT),
                    '-' => self.make_token(TokenType::TOKEN_MINUS),
                    '+' => self.make_token(TokenType::TOKEN_PLUS),
                    '/' => self.make_token(TokenType::TOKEN_SLASH),
                    '*' => self.make_token(TokenType::TOKEN_STAR),
                    '!' => {
                        let x = if self.matchChar('=') { TokenType::TOKEN_BANG_EQUAL } else { TokenType::TOKEN_BANG };
                        self.make_token(x)
                    }
                    '='=> {
                        let x =if self.matchChar('=') { TokenType::TOKEN_EQUAL_EQUAL } else { TokenType::TOKEN_EQUAL_EQUAL };
                        self.make_token(x)
                    }
                    '<' => {
                        let x = if self.matchChar('=') { TokenType::TOKEN_LESS_EQUAL } else { TokenType::TOKEN_LESS_EQUAL };
                        self.make_token(x)
                    }
                    '>' => {
                        let x = if self.matchChar('=') { TokenType::TOKEN_LESS_EQUAL } else { TokenType::TOKEN_GREATER };
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
            TokenType::TOKEN_IDENTIFIER
        }
    }

    fn identifier_type(&self) -> TokenType {
        match self.chars[self.start] {
            'a' => self.check_keyword(1, 2, "nd", TokenType::TOKEN_AND),
            'c' => self.check_keyword(1, 4, "lass", TokenType::TOKEN_CLASS),
            'e' => self.check_keyword(1, 3, "lse", TokenType::TOKEN_ELSE),
            'i' => self.check_keyword(1, 1, "f", TokenType::TOKEN_IF),
            'f' if self.current - self.start > 1 => {
                match self.chars[self.start + 1] {
                        'a'=> return self.check_keyword(2, 3, "lse", TokenType::TOKEN_FALSE),
                        'o'=> return self.check_keyword(2, 1, "r", TokenType::TOKEN_FOR),
                        'u'=> return self.check_keyword(2, 1, "n", TokenType::TOKEN_FUN),
                        _ => TokenType::TOKEN_IDENTIFIER
                    }
                }
            'n' => self.check_keyword(1, 2, "il", TokenType::TOKEN_NIL),
            'o' => self.check_keyword(1, 1, "r", TokenType::TOKEN_OR),
            'p' => self.check_keyword(1, 4, "rint", TokenType::TOKEN_PRINT),
            'r' => self.check_keyword(1, 5, "eturn", TokenType::TOKEN_RETURN),
            's' => self.check_keyword(1, 4, "uper", TokenType::TOKEN_SUPER),
            't' if self.current - self.start > 1 => {
                match self.chars[self.start + 1] {
                        'h'=> return self.check_keyword(2, 2, "is", TokenType::TOKEN_THIS),
                        'r'=> return self.check_keyword(2, 2, "ue", TokenType::TOKEN_TRUE),
                        _ => TokenType::TOKEN_IDENTIFIER
                    }
                }
            'v' => self.check_keyword(1, 2, "ar", TokenType::TOKEN_VAR),
            'w' => self.check_keyword(1, 4, "hile", TokenType::TOKEN_WHILE),
            _ => TokenType::TOKEN_IDENTIFIER,
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
      
        return self.make_token(TokenType::TOKEN_NUMBER);
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
        return self.make_token(TokenType::TOKEN_STRING);
    }

    fn make_eof(&self) -> Token {
        Token {
            tpe:     TokenType::TOKEN_EOF,
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
            tpe: TokenType::TOKEN_ERROR,
            text: String::from(message),
            line: self.line
        }
      }
}
