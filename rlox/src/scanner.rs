use std::str::Chars;
use std::vec::IntoIter;
use std::iter::Peekable;

pub struct Scanner {
    source: String,
    chars: Peekable<IntoIter<char>>,
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
            chars: source.chars().collect::<Vec<_>>().into_iter().peekable(),
            start: 0,
            current: 0,
            line: 1
        }
    }

    // mut to make equality happy ?
    fn matchChar(&mut self, expected: char) -> bool {
        match self.chars.peek() {
            None => false,
            Some(c) => if *c == expected {
                self.chars.next();
                true
            } else {
                false
            }
        }
    }


    // mut to make equality in match happy ?
    pub fn scan(&mut self) -> Token {
        self.start = self.current;
        // if self.is_at_end() {
        //     return self.make_token(TokenType::TOKEN_EOF);
        // }

        let c = self.chars.next();

        match c {
            None => self.make_token(TokenType::TOKEN_EOF),
            Some(c) =>
                match c {
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
                    _   => self.error_token("Unexpected character.")
                }
        }


    }

    fn make_token(&self, tpe: TokenType) -> Token {
        Token {
            tpe: tpe,
            text: String::from(&self.source[self.start..(self.current - self.start)]),
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