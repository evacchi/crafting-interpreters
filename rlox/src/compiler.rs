use std::cmp::PartialOrd;
use std::rc::Rc;

use chunk::Chunk;
use chunk::OpCode;

use memory::Memory;
use object::ObjType;

use scanner::Scanner;
use scanner::Token;
use scanner::TokenType;

use value::Value;

#[derive(PartialEq,PartialOrd)]
enum Precedence {
    None,
    Assignment,  // =
    Or,          // or
    And,         // and
    Equality,    // == !=
    Comparison,  // < > <= >=
    Term,        // + -
    Factor,      // * /
    Unary,       // ! -
    Call,        // . ()
    Primary
}

impl Precedence {
    fn from_index(index: usize) -> Option<Precedence> {
        match index {
            0  => Some(Precedence::None),
            1  => Some(Precedence::Assignment), 
            2  => Some(Precedence::Or),         
            3  => Some(Precedence::And),        
            4  => Some(Precedence::Equality),   
            5  => Some(Precedence::Comparison), 
            6  => Some(Precedence::Term),       
            7  => Some(Precedence::Factor),     
            8  => Some(Precedence::Unary),      
            9  => Some(Precedence::Call),       
            10 => Some(Precedence::Primary),
            _  => None
        }
    }
}


type BinaryRule = fn(&mut Parser) -> ();

type UnaryRule = fn(&mut Parser) -> ();

struct ParseRule{
    prefix: BinaryRule,
    infix: UnaryRule, 
    precedence: Precedence
}

struct Parser {
    scanner: Scanner,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
    emitter: BytecodeEmitter
}

pub struct Compiler {
    parser: Parser,
}

impl Compiler {
    pub fn new(source: String) -> Compiler {
        Compiler {
            parser: Parser::new(source)
        }
    }
    pub fn compile(&mut self) -> bool {
        self.parser.advance();

        while !self.parser.matches(TokenType::Eof) {
            self.parser.declaration();
        }

        self.parser.end(self.parser.previous.line);
        !self.parser.had_error
    }
    pub fn state(self) -> (Chunk, Memory) {
        (self.parser.emitter.current_chunk, self.parser.emitter.memory)
    }
}

struct BytecodeEmitter {
    current_chunk: Chunk,
    memory: Memory
}

impl BytecodeEmitter {
    pub fn new() -> BytecodeEmitter { 
        BytecodeEmitter {
            current_chunk : Chunk::new(),
            memory : Memory::new()
        }
    }

    pub fn emit_byte(&mut self, op: OpCode, line: usize) {
        self.current_chunk.write(op, line);
    }

    pub fn emit_bytes(&mut self, op1: OpCode, op2: OpCode, line: usize) {
        self.current_chunk.write(op1, line);
        self.current_chunk.write(op2, line);
    }


    pub fn emit_return(&mut self, line: usize) {
        self.emit_byte(OpCode::Return, line);
    }
    pub fn emit_constant(&mut self, value: Value, line: usize) {
        if let Value::Object(o) = &value {
            self.memory.push(o.clone());
        }

        let index = self.current_chunk.write_constant(value);
        self.emit_byte(OpCode::Constant{ index }, line);
    }

}

impl ParseRule {
    fn new(prefix: BinaryRule, infix: UnaryRule, precedence: Precedence) -> ParseRule {
        ParseRule { prefix, infix, precedence }
    }


    fn of_token(tpe: &TokenType) -> ParseRule {
        match tpe {
            TokenType::Start        => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::LeftParen    => ParseRule::new(Parser::grouping, Parser::err,      Precedence::None),
            TokenType::RightParen   => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::LeftBrace    => ParseRule::new(Parser::err,      Parser::err,      Precedence::None), 
            TokenType::RightBrace   => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::Comma        => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::Dot          => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::Minus        => ParseRule::new(Parser::unary,    Parser::binary,   Precedence::Term),
            TokenType::Plus         => ParseRule::new(Parser::err,      Parser::binary,   Precedence::Term),
            TokenType::Semicolon    => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::Slash        => ParseRule::new(Parser::err,      Parser::binary,   Precedence::Factor),
            TokenType::Star         => ParseRule::new(Parser::err,      Parser::binary,   Precedence::Factor),
            TokenType::Bang         => ParseRule::new(Parser::unary,    Parser::err,      Precedence::None),
            TokenType::BangEqual    => ParseRule::new(Parser::err,      Parser::binary,   Precedence::Equality),
            TokenType::Equal        => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::EqualEqual   => ParseRule::new(Parser::err,      Parser::binary,   Precedence::Comparison),
            TokenType::Greater      => ParseRule::new(Parser::err,      Parser::binary,   Precedence::Comparison),
            TokenType::GreaterEqual => ParseRule::new(Parser::err,      Parser::binary,   Precedence::Comparison),
            TokenType::Less         => ParseRule::new(Parser::err,      Parser::binary,   Precedence::Comparison),
            TokenType::LessEqual    => ParseRule::new(Parser::err,      Parser::binary,   Precedence::Comparison),
            TokenType::Identifier   => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::String       => ParseRule::new(Parser::string,   Parser::err,      Precedence::None),
            TokenType::Number       => ParseRule::new(Parser::number,   Parser::err,      Precedence::None),
            TokenType::And          => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::Class        => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::Else         => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::False        => ParseRule::new(Parser::literal,  Parser::err,      Precedence::None),
            TokenType::For          => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::Fun          => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::If           => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::Nil          => ParseRule::new(Parser::literal,  Parser::err,      Precedence::None),
            TokenType::Or           => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::Print        => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::Return       => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::Super        => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::This         => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::True         => ParseRule::new(Parser::literal,  Parser::err,      Precedence::None),
            TokenType::Var          => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::While        => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::Error        => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
            TokenType::Eof          => ParseRule::new(Parser::err,      Parser::err,      Precedence::None),
        }
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
            emitter: BytecodeEmitter::new()

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

    fn matches(&mut self, tpe: TokenType) -> bool {
        if !self.check(tpe) {
            false
        } else {
            self.advance();
            true
        }
    }

    fn check(&mut self, tpe: TokenType) -> bool {
        self.current.tpe == tpe
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
      }

    pub fn number(&mut self) {
        let n = self.previous.text.parse::<f64>().unwrap();
        self.emitter.emit_constant(Value::Number(n), self.previous.line);
    }

    fn string(&mut self) {
        let value = Value::Object(ObjType::String(Rc::new(self.previous.text.clone())));
        self.emitter.emit_constant(value, self.previous.line);
    }

    fn unary(&mut self) {
        let tok = self.previous.clone();
      
        // Compile the operand.
        self.parse_precedence(Precedence::Unary);
      
        // Emit the operator instruction.
        match tok.tpe { 
            TokenType::Bang => self.emitter.emit_byte(OpCode::Not, self.previous.line),
            TokenType::Minus => self.emitter.emit_byte(OpCode::Negate, self.previous.line),
            _ => {} // Unreachable.
        }
    }
      
    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = ParseRule::of_token(&self.previous.tpe).prefix;      
        prefix_rule(self);

        while precedence <= ParseRule::of_token(&self.current.tpe).precedence {
            self.advance();
            let infix_rule = ParseRule::of_token(&self.previous.tpe).infix;
            infix_rule(self);
          }
        
    }

    pub fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment)
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emitter.emit_byte(OpCode::Print, self.current.line);
    }

    pub fn declaration(&mut self) {
        self.statement()
    }

    pub fn statement(&mut self) {
        if self.matches(TokenType::Print) {
            self.print_statement();
        }
    }

    pub fn end(&mut self, line: usize) {
        self.emitter.emit_return(line);
        // debug statements
        if !self.had_error {
            self.emitter.current_chunk.disassemble("code");
        }        
    }

    pub fn binary(&mut self) {
        // Remember the operator.
        let tok = self.previous.clone();
        let line = self.previous.line;
      
        // Compile the right operand.
        let rule = ParseRule::of_token(&tok.tpe);
        let next_rule = rule.precedence as usize + 1;
        let next_prec = Precedence::from_index(next_rule)
                            .expect("No match for given index");
        self.parse_precedence(next_prec);

        // Emit the operator instruction.
        match tok.tpe {
            TokenType::BangEqual    => self.emitter.emit_bytes(OpCode::Equal, OpCode::Not, line),
            TokenType::EqualEqual   => self.emitter.emit_byte(OpCode::Equal, line),
            TokenType::Greater      => self.emitter.emit_byte(OpCode::Greater, line),
            TokenType::GreaterEqual => self.emitter.emit_bytes(OpCode::Less, OpCode::Not, line),
            TokenType::Less         => self.emitter.emit_byte(OpCode::Less, line),
            TokenType::LessEqual    => self.emitter.emit_bytes(OpCode::Greater, OpCode::Not, line),
            TokenType::Plus         => self.emitter.emit_byte(OpCode::Add, line),
            TokenType::Minus        => self.emitter.emit_byte(OpCode::Subtract, line),
            TokenType::Star         => self.emitter.emit_byte(OpCode::Multiply, line),
            TokenType::Slash        => self.emitter.emit_byte(OpCode::Divide, line),
          _ => {}// Unreachable.
        }
    }

    pub fn literal(&mut self) {
        let tok = self.previous.clone();
        match tok.tpe {
            TokenType::False => self.emitter.emit_byte(OpCode::False, tok.line),
            TokenType::Nil   => self.emitter.emit_byte(OpCode::Nil,   tok.line),
            TokenType::True  => self.emitter.emit_byte(OpCode::True,  tok.line),
            _ => {} // Unreachable.
        }
    }

    fn err(&mut self) {
        self.error("Expect expression.");
    }

    pub fn error(&mut self, message: &str) {
        self.error_at(self.previous.clone(), message);
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