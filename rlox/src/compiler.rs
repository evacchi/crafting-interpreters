use std::cmp::PartialOrd;

use chunk::Chunk;
use chunk::OpCode;

use memory::Memory;
use object::ObjType;

use scanner::Scanner;
use scanner::Token;
use scanner::TokenType;

use value::Value;

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl Precedence {
    fn from_index(index: usize) -> Option<Precedence> {
        match index {
            0 => Some(Precedence::None),
            1 => Some(Precedence::Assignment),
            2 => Some(Precedence::Or),
            3 => Some(Precedence::And),
            4 => Some(Precedence::Equality),
            5 => Some(Precedence::Comparison),
            6 => Some(Precedence::Term),
            7 => Some(Precedence::Factor),
            8 => Some(Precedence::Unary),
            9 => Some(Precedence::Call),
            10 => Some(Precedence::Primary),
            _ => None,
        }
    }
}

type BinaryRule = fn(&mut Parser, bool) -> ();

type UnaryRule = fn(&mut Parser, bool) -> ();

struct ParseRule {
    prefix: BinaryRule,
    infix: UnaryRule,
    precedence: Precedence,
}

struct Parser {
    scanner: Scanner,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
    emitter: BytecodeEmitter,
    scope: Scope,
}

#[derive(Debug, Clone)]
struct Local {
    name: Token,
    depth: i32,
}

struct Scope {
    locals: Vec<Local>,
    depth: i32,
}

pub struct Compiler {
    parser: Parser,
}

impl Scope {
    fn new() -> Scope {
        Scope {
            locals: Vec::new(),
            depth: 0,
        }
    }

    fn begin(&mut self) {
        self.depth += 1;
    }

    fn end(&mut self) -> i32 {
        let mut count = 0;
        while self.locals.len() > 0 && self.locals.last().unwrap().depth > self.depth {
            count += 1;
        }
        self.depth -= 1;
        count
    }

    fn add_local(&mut self, name: Token) {
        let local = Local { name, depth: -1 };
        self.locals.push(local);
    }

    fn mark_initialized(&mut self) {
        let lastidx = self.locals.len() - 1;
        self.locals[lastidx].depth = self.depth;
    }

    fn resolve_local(&mut self, name: &Token) -> Result<Option<usize>, &'static str> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if name.text == local.name.text {
                if local.depth == -1 {
                    return Err("Can't read local variable in its own initializer.");
                } else {
                    return Ok(Some(i));
                }
            }
        }
        return Ok(None);
    }
}

impl Compiler {
    pub fn new(source: String) -> Compiler {
        Compiler {
            parser: Parser::new(source),
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
        (
            self.parser.emitter.current_chunk,
            self.parser.emitter.memory,
        )
    }
}

struct BytecodeEmitter {
    current_chunk: Chunk,
    memory: Memory,
}

impl BytecodeEmitter {
    pub fn new() -> BytecodeEmitter {
        BytecodeEmitter {
            current_chunk: Chunk::new(),
            memory: Memory::new(),
        }
    }

    pub fn emit_byte(&mut self, op: OpCode, line: usize) {
        self.current_chunk.write(op, line);
    }

    pub fn emit_bytes(&mut self, op1: OpCode, op2: OpCode, line: usize) {
        self.current_chunk.write(op1, line);
        self.current_chunk.write(op2, line);
    }

    // pub fn emit_loop(&mut self, loopStart) {
    //     self.emit_byte(OpCode::Loop, );

    //     int offset = currentChunk()->count - loopStart + 2;
    //     if (offset > UINT16_MAX) error("Loop body too large.");

    //     emitByte((offset >> 8) & 0xff);
    //     emitByte(offset & 0xff);
    //   }

    pub fn emit_return(&mut self, line: usize) {
        self.emit_byte(OpCode::Return, line);
    }

    pub fn write_constant(&mut self, value: Value) -> usize {
        self.current_chunk.write_constant(value)
    }

    pub fn emit_constant(&mut self, value: Value, line: usize) -> usize {
        if let Value::Object(o) = &value {
            self.memory.push(o.clone());
        }

        let index = self.current_chunk.write_constant(value);
        self.emit_byte(OpCode::Constant { index }, line);
        index
    }

    pub fn patch_jump(&mut self, offset: usize) {
        let new_jump = self.current_chunk.code.len() - 1 - offset;
        let new_op = match self.current_chunk.code[offset] {
            OpCode::JumpIfFalse { jump: _ } => OpCode::JumpIfFalse { jump: new_jump },
            OpCode::Jump { jump: _ } => OpCode::Jump { jump: new_jump },
            op => panic!("Expected a Jump instruction! Found {:?}", op),
        };
        self.current_chunk.code[offset] = new_op;
    }
}

impl ParseRule {
    fn new(prefix: BinaryRule, infix: UnaryRule, precedence: Precedence) -> ParseRule {
        ParseRule {
            prefix,
            infix,
            precedence,
        }
    }

    fn of_token(tpe: &TokenType) -> ParseRule {
        match tpe {
            TokenType::Start => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::LeftParen => ParseRule::new(Parser::grouping, Parser::err, Precedence::None),
            TokenType::RightParen => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::LeftBrace => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::RightBrace => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::Comma => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::Dot => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::Minus => ParseRule::new(Parser::unary, Parser::binary, Precedence::Term),
            TokenType::Plus => ParseRule::new(Parser::err, Parser::binary, Precedence::Term),
            TokenType::Semicolon => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::Slash => ParseRule::new(Parser::err, Parser::binary, Precedence::Factor),
            TokenType::Star => ParseRule::new(Parser::err, Parser::binary, Precedence::Factor),
            TokenType::Bang => ParseRule::new(Parser::unary, Parser::err, Precedence::None),
            TokenType::BangEqual => {
                ParseRule::new(Parser::err, Parser::binary, Precedence::Equality)
            }
            TokenType::Equal => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::EqualEqual => {
                ParseRule::new(Parser::err, Parser::binary, Precedence::Comparison)
            }
            TokenType::Greater => {
                ParseRule::new(Parser::err, Parser::binary, Precedence::Comparison)
            }
            TokenType::GreaterEqual => {
                ParseRule::new(Parser::err, Parser::binary, Precedence::Comparison)
            }
            TokenType::Less => ParseRule::new(Parser::err, Parser::binary, Precedence::Comparison),
            TokenType::LessEqual => {
                ParseRule::new(Parser::err, Parser::binary, Precedence::Comparison)
            }
            TokenType::Identifier => {
                ParseRule::new(Parser::variable, Parser::err, Precedence::None)
            }
            TokenType::String => ParseRule::new(Parser::string, Parser::err, Precedence::None),
            TokenType::Number => ParseRule::new(Parser::number, Parser::err, Precedence::None),
            TokenType::And => ParseRule::new(Parser::err, Parser::and_, Precedence::And),
            TokenType::Class => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::Else => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::False => ParseRule::new(Parser::literal, Parser::err, Precedence::None),
            TokenType::For => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::Fun => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::If => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::Nil => ParseRule::new(Parser::literal, Parser::err, Precedence::None),
            TokenType::Or => ParseRule::new(Parser::err, Parser::or_, Precedence::Or),
            TokenType::Print => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::Return => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::Super => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::This => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::True => ParseRule::new(Parser::literal, Parser::err, Precedence::None),
            TokenType::Var => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::While => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::Error => ParseRule::new(Parser::err, Parser::err, Precedence::None),
            TokenType::Eof => ParseRule::new(Parser::err, Parser::err, Precedence::None),
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
                line: 0,
            },
            previous: Token {
                tpe: TokenType::Start,
                text: String::from(""),
                line: 0,
            },
            had_error: false,
            panic_mode: false,
            emitter: BytecodeEmitter::new(),
            scope: Scope::new(),
        }
    }

    pub fn advance(&mut self) {
        self.previous = self.current.clone();

        loop {
            self.current = self.scanner.scan();
            if self.current.tpe != TokenType::Error {
                break;
            }

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

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    pub fn number(&mut self, _can_assign: bool) {
        let n = self.previous.text.parse::<f64>().unwrap();
        self.emitter
            .emit_constant(Value::Number(n), self.previous.line);
    }

    fn or_(&mut self, _can_assign: bool) {
        self.emitter
            .emit_byte(OpCode::JumpIfFalse { jump: 0xFF }, self.current.line);
        let else_jump = self.emitter.current_chunk.code.len() - 1;
        self.emitter
            .emit_byte(OpCode::Jump { jump: 0xFF }, self.current.line);
        let end_jump = self.emitter.current_chunk.code.len() - 1;
        self.emitter.patch_jump(else_jump);
        self.emitter.emit_byte(OpCode::Pop, self.current.line);
        self.parse_precedence(Precedence::Or);
        self.emitter.patch_jump(end_jump);
    }

    fn string(&mut self, _can_assign: bool) {
        let value = Value::Object(ObjType::String(self.previous.text.clone()));
        self.emitter.emit_constant(value, self.previous.line);
    }

    fn named_variable(&mut self, name: &Token, can_assign: bool) {
        let result = self.scope.resolve_local(name);
        if can_assign && self.matches(TokenType::Equal) {
            self.expression();
            match result {
                Ok(optindex) => {
                    let op = match optindex {
                        Some(index) => OpCode::SetLocal { index },
                        None => OpCode::SetGlobal {
                            index: self.identifier_constant(name),
                        },
                    };
                    self.emitter.emit_byte(op, self.current.line);
                }
                Err(msg) => self.error(msg),
            }
        } else {
            match result {
                Ok(optindex) => {
                    let op = match optindex {
                        Some(index) => OpCode::GetLocal { index },
                        None => OpCode::GetGlobal {
                            index: self.identifier_constant(name),
                        },
                    };
                    self.emitter.emit_byte(op, self.current.line);
                }
                Err(msg) => self.error(msg),
            }
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(&self.previous.clone(), can_assign)
    }

    fn unary(&mut self, _can_assign: bool) {
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

        let can_assign = precedence <= Precedence::Assignment;
        prefix_rule(self, can_assign);

        while precedence <= ParseRule::of_token(&self.current.tpe).precedence {
            self.advance();
            let infix_rule = ParseRule::of_token(&self.previous.tpe).infix;
            infix_rule(self, can_assign);
        }

        if can_assign && self.matches(TokenType::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    fn identifier_constant(&mut self, name: &Token) -> usize {
        self.emitter
            .write_constant(Value::Object(ObjType::String(name.text.clone())))
    }

    fn declare_variable(&mut self) {
        if self.scope.depth == 0 {
            return;
        }
        let name = self.previous.clone();
        let iter = &self.scope.locals.clone();
        for local in iter.iter().rev() {
            if local.depth != -1 && local.depth < self.scope.depth {
                break;
            }
            if name.text == local.name.text {
                self.error("Already variable with this name in this scope.");
            }
        }

        self.scope.add_local(name);
    }

    fn parse_variable(&mut self, err: &str) -> usize {
        self.consume(TokenType::Identifier, err);

        self.declare_variable();
        if self.scope.depth > 0 {
            return 0;
        }

        self.identifier_constant(&self.previous.clone())
    }

    fn define_variable(&mut self, index: usize) {
        if self.scope.depth > 0 {
            self.scope.mark_initialized();
            return;
        }

        self.emitter
            .emit_byte(OpCode::DefineGlobal { index }, self.current.line);
    }

    fn and_(&mut self, _can_assign: bool) {
        self.emitter
            .emit_byte(OpCode::JumpIfFalse { jump: 0xFF }, self.current.line);
        let end_jump = self.emitter.current_chunk.code.len() - 1;
        self.emitter.emit_byte(OpCode::Pop, self.current.line);
        self.parse_precedence(Precedence::And);
        self.emitter.patch_jump(end_jump);
    }

    pub fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment)
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.matches(TokenType::Equal) {
            self.expression();
        } else {
            self.emitter.emit_byte(OpCode::Nil, self.current.line);
        }
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );

        self.define_variable(global);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emitter.emit_byte(OpCode::Pop, self.current.line);
    }

    /*
    if (match(TOKEN_SEMICOLON)) {
      // No initializer.
    } else if (match(TOKEN_VAR)) {
      varDeclaration();
    } else {
      expressionStatement();
    }

    int loopStart = currentChunk()->count;

      */

    fn for_statement(&mut self) {
        self.scope.begin();

        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");
        if self.matches(TokenType::Semicolon) {
            // No inizializer.
        } else if self.matches(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let loop_start = self.emitter.current_chunk.code.len() - 1;

        let mut exit_jump = None;

        if !self.matches(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

            // Jump out of the loop if the condition is false.
            self.emitter
                .emit_byte(OpCode::JumpIfFalse { jump: 0xFF }, self.current.line);
            exit_jump = Some(self.emitter.current_chunk.code.len() - 1);

            self.emitter.emit_byte(OpCode::Pop, self.current.line); // Condition.
        }

        self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

        self.statement();
        let jump = self.emitter.current_chunk.code.len() - loop_start;
        self.emitter
            .emit_byte(OpCode::Loop { jump }, self.current.line);

        if let Some(jump) = exit_jump {
            self.emitter.patch_jump(jump);
            self.emitter.emit_byte(OpCode::Pop, self.current.line); // Condition.
        }

        self.scope.end();
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        self.emitter
            .emit_byte(OpCode::JumpIfFalse { jump: 0xFF }, self.current.line);
        let then_jump = self.emitter.current_chunk.code.len() - 1;
        self.emitter.emit_byte(OpCode::Pop, self.current.line);
        self.statement();

        self.emitter
            .emit_byte(OpCode::Jump { jump: 0xFF }, self.current.line);
        let else_jump = self.emitter.current_chunk.code.len() - 1;

        self.emitter.patch_jump(then_jump);
        self.emitter.emit_byte(OpCode::Pop, self.current.line);

        if self.matches(TokenType::Else) {
            self.statement();
        }

        self.emitter.patch_jump(else_jump)
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emitter.emit_byte(OpCode::Print, self.current.line);
    }

    fn while_statement(&mut self) {
        let loop_start = self.emitter.current_chunk.code.len() - 1;
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        self.emitter
            .emit_byte(OpCode::JumpIfFalse { jump: 0xFF }, self.current.line);
        let exit_jump = self.emitter.current_chunk.code.len() - 1;

        self.emitter.emit_byte(OpCode::Pop, self.current.line);
        self.statement();

        let jump = self.emitter.current_chunk.code.len() - loop_start;
        self.emitter
            .emit_byte(OpCode::Loop { jump }, self.current.line);

        self.emitter.patch_jump(exit_jump);
        self.emitter.emit_byte(OpCode::Pop, self.current.line);
    }

    fn synchronize(&mut self) {
        self.panic_mode = true;

        while self.current.tpe != TokenType::Eof {
            if self.previous.tpe == TokenType::Semicolon {
                return;
            }
            match self.current.tpe {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {}
            }

            self.advance();
        }
    }

    pub fn declaration(&mut self) {
        if self.matches(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    pub fn statement(&mut self) {
        if self.matches(TokenType::Print) {
            self.print_statement();
        } else if self.matches(TokenType::For) {
            self.for_statement();
        } else if self.matches(TokenType::If) {
            self.if_statement();
        } else if self.matches(TokenType::While) {
            self.while_statement();
        } else if self.matches(TokenType::LeftBrace) {
            self.scope.begin();
            self.block();
            for _ in 0..self.scope.end() {
                self.emitter.emit_byte(OpCode::Pop, self.current.line);
            }
        } else {
            self.expression_statement();
        }
    }

    pub fn end(&mut self, line: usize) {
        self.emitter.emit_return(line);
        // debug statements
        if !self.had_error {
            self.emitter.current_chunk.disassemble("code");
        }
    }

    pub fn binary(&mut self, _can_assign: bool) {
        // Remember the operator.
        let tok = self.previous.clone();
        let line = self.previous.line;

        // Compile the right operand.
        let rule = ParseRule::of_token(&tok.tpe);
        let next_rule = rule.precedence as usize + 1;
        let next_prec = Precedence::from_index(next_rule).expect("No match for given index");
        self.parse_precedence(next_prec);

        // Emit the operator instruction.
        match tok.tpe {
            TokenType::BangEqual => self.emitter.emit_bytes(OpCode::Equal, OpCode::Not, line),
            TokenType::EqualEqual => self.emitter.emit_byte(OpCode::Equal, line),
            TokenType::Greater => self.emitter.emit_byte(OpCode::Greater, line),
            TokenType::GreaterEqual => self.emitter.emit_bytes(OpCode::Less, OpCode::Not, line),
            TokenType::Less => self.emitter.emit_byte(OpCode::Less, line),
            TokenType::LessEqual => self.emitter.emit_bytes(OpCode::Greater, OpCode::Not, line),
            TokenType::Plus => self.emitter.emit_byte(OpCode::Add, line),
            TokenType::Minus => self.emitter.emit_byte(OpCode::Subtract, line),
            TokenType::Star => self.emitter.emit_byte(OpCode::Multiply, line),
            TokenType::Slash => self.emitter.emit_byte(OpCode::Divide, line),
            _ => {} // Unreachable.
        }
    }

    pub fn literal(&mut self, _can_assign: bool) {
        let tok = self.previous.clone();
        match tok.tpe {
            TokenType::False => self.emitter.emit_byte(OpCode::False, tok.line),
            TokenType::Nil => self.emitter.emit_byte(OpCode::Nil, tok.line),
            TokenType::True => self.emitter.emit_byte(OpCode::True, tok.line),
            _ => {} // Unreachable.
        }
    }

    fn err(&mut self, _can_assign: bool) {
        self.error("Expect expression.");
    }

    pub fn error(&mut self, message: &str) {
        self.error_at(self.previous.clone(), message);
    }

    pub fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current.clone(), message);
    }

    pub fn error_at(&mut self, token: Token, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;

        eprint!("[line {}] Error", token.line);

        if token.tpe == TokenType::Eof {
            eprint!(" at end");
        } else if token.tpe == TokenType::Error {
            // Nothing.
        } else {
            eprint!(" at '{}'", token.text);
        }

        eprint!(": {}\n", message);
        self.had_error = true;
    }
}
