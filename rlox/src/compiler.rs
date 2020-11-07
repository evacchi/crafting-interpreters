use scanner::Scanner;
use scanner::Token;
use scanner::TokenType;

use chunk::Chunk;
use chunk::OpCode;

struct Parser {
    scanner: Scanner,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool
}

pub struct Compiler {
    parser: Parser,
    current_chunk: Chunk
}

impl Compiler {
    pub fn new(source: String) -> Compiler {
        Compiler {
            parser: Parser::new(source),
            current_chunk: Chunk::new()
        }
    }
    pub fn compile(&mut self) -> bool {
        self.parser.advance();
        self.parser.expression();
        self.parser.consume(TokenType::Eof, "Expect end of expression.");
        self.end();
        !self.parser.had_error
    }
    pub fn emit_byte(&mut self, op: OpCode) {
        self.current_chunk.write(op, self.parser.previous.line);
    }
    pub fn end(&mut self) {
        self.emit_return();
    }
    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }
      
}

impl Parser {
    pub fn new(source: String) -> Parser {
        Parser {
            scanner: Scanner::new(source),
            current: Token {
                tpe: TokenType::Start,
                text: String::from(""),
                line: 0
            },
            previous: Token {
                tpe: TokenType::Start,
                text: String::from(""),
                line: 0
            },
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn advance(&mut self) {
        self.previous = self.current.clone();
    
        loop {
            self.current = self.scanner.scan();
            if self.current.tpe != TokenType::Error { break;}
        
            self.error_at_current(&self.current.text.clone());
        }
    }

    pub fn consume(&mut self, tpe: TokenType, message: &str) {
        if self.current.tpe == tpe {
            self.advance();
            return;
        }
        
        self.error_at_current(message);
    }

    pub fn expression(&self) {

    }


    pub fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current.clone(), message);

    }

    pub fn error_at(&mut self, token: Token, message: &str) {
        if self.panic_mode { return; }
        self.panic_mode = true;
      
        eprint!("[line {}] Error", token.line);
      
        if token.tpe == TokenType::Eof {
            eprint!(" at end");
        } else if token.tpe == TokenType::Error {
          // Nothing.
        } else {
            eprint!( " at '{}'", token.text);
        }
      
        eprint!( ": {}\n", message);
        self.had_error = true;
      }
      

}