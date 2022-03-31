use std::cmp::PartialOrd;

use chunk::Chunk;
use chunk::OpCode;

use memory::Memory;
use object::Function;
use object::FunctionType;
use object::ObjType;
use object::Upvalue;

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

pub struct Parser {
    scanner: Scanner,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
    scope: Scope,
}

#[derive(Debug, Clone)]
struct Local {
    name: Token,
    depth: i32,
    is_captured: bool
}

#[derive(Clone)]
struct ScopeCell {
    locals: Vec<Local>,
    upvalues: Vec<Upvalue>,
    depth: i32,
    emitter: BytecodeEmitter,
}

impl ScopeCell {

    fn new() -> ScopeCell {
        ScopeCell {
            locals: vec![
                Local {
                    name: Token {
                        tpe: TokenType::Undefined,
                        line: 0,
                        text: String::from("")
                    },
                    depth: 0,
                    is_captured: false
                }
            ],
            upvalues: Vec::new(),
            depth: 0,
            emitter: BytecodeEmitter::new(),
        }
    }

    fn add_upvalue(&mut self, index: usize, is_local: bool) -> usize {
        let upvalue = self.upvalues
            .iter().enumerate()
            .find(|(_i,u)| u.index == index && u.is_local == is_local);
            match upvalue {
                Some((i,_)) => i,
                None => {
                    self.upvalues.push(Upvalue{index,is_local});
                    self.upvalues.len() - 1
                } 
            }
    }

    fn add_local(&mut self, name: Token) {
        let local = Local { name, depth: -1, is_captured: false };
        self.locals.push(local);
    }

    fn mark_initialized(&mut self) {
        if self.depth == 0 {
            return;
        }
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

    fn depth(&mut self) -> i32 {
        self.depth
    }

    fn locals(&mut self) -> &mut Vec<Local> {
        &mut self.locals
    }

    fn begin(&mut self) {
        self.depth += 1;
    }

    fn end(&mut self, line: usize) -> i32 {
        self.depth -= 1;

        let mut count: i32 = 0;
        while self.locals().len() > 0 && self.locals().last().unwrap().depth > self.depth() {
            if self.locals()[count as usize].is_captured {
                self.emitter.emit_byte(OpCode::CloseUpvalue, line);
            } else {
                self.emitter.emit_byte(OpCode::Pop, line);
            }
            self.locals().pop();
            count += 1;
        }
        count
    }
}

struct Scope {
    pub stack: Vec<ScopeCell>,
}


pub struct Compiler<'a> {
    parser: &'a mut Parser,
}

impl Scope {
    fn new() -> Scope {
        Scope {
            stack: vec! [ ScopeCell::new() ]
        }
    }

    fn resolve_upvalue(&mut self, name: &Token) -> Result<Option<usize>, &'static str> {
        // there is no enclosing function: assume global
        if self.stack.len() == 1 {
            return Ok(None);
        }

        // look for  a local in the enclosing function
        let l = self.stack.len();
        let enclosing = &mut self.stack[l-2];
        match enclosing.resolve_local(name)? {
            Some(local) => {
                enclosing.locals()[local].is_captured = true;
                Ok(Some(self.stack.last_mut().unwrap().add_upvalue(local, true)))
            }
            None => {
                match self.resolve_upvalue(name)? {
                    Some(upvalue) => 
                        Ok(Some(self.stack.last_mut().unwrap().add_upvalue(upvalue, false))),
                    None => Ok(None)
                }
            }
        }
        //self.stack.last_mut().unwrap().resolve_local(name)
    }

    fn resolve_local(&mut self, name: &Token) -> Result<Option<usize>, &'static str> {
        self.stack.last_mut().unwrap().resolve_local(name)
    }

}

impl <'a> Compiler<'a> {
    pub fn new(parser: &'a mut Parser) -> Compiler<'a> {
        Compiler {
            parser
        }
    }
    pub fn compile(&mut self) -> Option<Function> {
        self.parser.advance();

        while !self.parser.matches(TokenType::Eof) {
            self.parser.declaration();
        }

        if self.parser.had_error {
            None
        } else {
            self.parser.end(self.parser.previous.line);
            let f = self.parser.emitter().function.clone();
            Some(f)
        }
    }
    pub fn state(self) -> BytecodeEmitter {
        self.parser.emitter().clone()
    }
}

#[derive(Clone)]
pub struct BytecodeEmitter {
    pub function: Function,
    pub memory: Memory,
}

impl BytecodeEmitter {
    pub fn new() -> BytecodeEmitter {
        BytecodeEmitter {
            function: Function::main(),
            memory: Memory::new(),
        }
    }

    pub fn chunk(&mut self) -> &mut Chunk {
        &mut self.function.chunk
    }

    pub fn emit_byte(&mut self, op: OpCode, line: usize) {
        self.chunk().write(op, line);
    }

    pub fn emit_bytes(&mut self, op1: OpCode, op2: OpCode, line: usize) {
        self.chunk().write(op1, line);
        self.chunk().write(op2, line);
    }

    pub fn emit_return(&mut self, line: usize) {
        self.emit_byte(OpCode::Nil, line);
        self.emit_byte(OpCode::Return, line);
    }

    pub fn write_constant(&mut self, value: Value) -> usize {
        self.chunk().write_constant(value)
    }

    pub fn emit_constant(&mut self, value: Value, line: usize) -> usize {
        if let Value::Object(o) = &value {
            self.memory.push(o.clone());
        }

        let index = self.chunk().write_constant(value);
        self.emit_byte(OpCode::Constant { index }, line);
        index
    }

    pub fn patch_jump(&mut self, offset: usize) {
        let new_jump = self.chunk().code.len() - 1 - offset;
        let new_op = match self.chunk().code[offset] {
            OpCode::JumpIfFalse { jump: _ } => OpCode::JumpIfFalse { jump: new_jump },
            OpCode::Jump { jump: _ } => OpCode::Jump { jump: new_jump },
            op => panic!("Expected a Jump instruction! Found {:?}", op),
        };
        self.chunk().code[offset] = new_op;
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
            TokenType::LeftParen => ParseRule::new(Parser::grouping, Parser::call, Precedence::Call),
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
            _ => ParseRule::new(Parser::err, Parser::err, Precedence::None)
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
            scope: Scope::new(),
        }
    }

    pub fn scope(&mut self) -> &mut ScopeCell {
        self.scope.stack.last_mut().unwrap()
    }

    pub fn emitter(&mut self) -> &mut BytecodeEmitter {
        &mut self.scope.stack.last_mut().unwrap().emitter
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
        let l = self.previous.line;
        self.emitter()
            .emit_constant(Value::Number(n), l);
    }

    fn or_(&mut self, _can_assign: bool) {
        let l = self.current.line;
        self.emitter()
            .emit_byte(OpCode::JumpIfFalse { jump: 0xFF }, l);
        let else_jump = self.emitter().chunk().code.len() - 1;
        self.emitter()
            .emit_byte(OpCode::Jump { jump: 0xFF }, l);
        let end_jump = self.emitter().chunk().code.len() - 1;
        self.emitter().patch_jump(else_jump);
        self.emitter().emit_byte(OpCode::Pop, l);
        self.parse_precedence(Precedence::Or);
        self.emitter().patch_jump(end_jump);
    }

    fn string(&mut self, _can_assign: bool) {
        let l = self.previous.line;
        let value = Value::Object(ObjType::String(self.previous.text.clone()));
        self.emitter().emit_constant(value, l);
    }

    fn named_variable(&mut self, name: &Token, can_assign: bool) {
        let cons_get: fn (usize) -> OpCode;
        let cons_set: fn (usize) -> OpCode;

        let arg;
        match self.scope().resolve_local(name) {
            Err(msg) => {
                self.error(msg);
                return;
            }
            Ok(Some(arg_)) => {
                arg = arg_;
                cons_get = |index| OpCode::GetLocal { index };
                cons_set = |index| OpCode::SetLocal { index };
            }
            Ok(None) => {
                match self.scope.resolve_upvalue(name) {
                    Err(msg) => {
                        self.error(msg);
                        return;
                    }
                    Ok(Some(arg_)) => {
                        arg = arg_;
                        cons_get = |index| OpCode::GetUpvalue { index };
                        cons_set = |index| OpCode::SetUpvalue { index };
                    }
                    Ok(None) => {
                        arg = self.identifier_constant(name);
                        cons_get = |index| OpCode::GetGlobal { index };
                        cons_set = |index| OpCode::SetGlobal { index };
                    }
                }
            }
        }


        if can_assign && self.matches(TokenType::Equal) {
            self.expression();
            let l = self.current.line;
            self.emitter().emit_byte(cons_set(arg), l)
        } else {
            let l = self.current.line;
            self.emitter().emit_byte(cons_get(arg), l)
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
        let l = self.current.line;
        match tok.tpe {
            TokenType::Bang => self.emitter().emit_byte(OpCode::Not, l),
            TokenType::Minus => self.emitter().emit_byte(OpCode::Negate, l),
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
        self.emitter()
            .write_constant(Value::Object(ObjType::String(name.text.clone())))
    }

    fn declare_variable(&mut self) {
        if self.scope().depth() == 0 {
            return;
        }
        let name = self.previous.clone();
        let iter = &self.scope().locals().clone();
        for local in iter.iter().rev() {
            if local.depth != -1 && local.depth < self.scope().depth() {
                break;
            }
            if name.text == local.name.text {
                self.error("Already variable with this name in this scope.");
            }
        }

        self.scope().add_local(name);
    }

    fn parse_variable(&mut self, err: &str) -> usize {
        self.consume(TokenType::Identifier, err);

        self.declare_variable();
        if self.scope().depth() > 0 {
            return 0;
        }

        self.identifier_constant(&self.previous.clone())
    }

    fn define_variable(&mut self, index: usize) {
        if self.scope().depth() > 0 {
            self.scope().mark_initialized();
            return;
        }

        let l = self.current.line;
        self.emitter()
            .emit_byte(OpCode::DefineGlobal { index }, l);
    }

    fn argument_list(&mut self) -> u32 {
        let mut argc = 0;
        if !self.check(TokenType::RightParen) {
            loop {
                self.expression();
                argc += 1;
                if !self.matches(TokenType::Comma) { break; }
            } 
        }
        self.consume(TokenType::RightParen, "Expect ')' after arguments.");
        return argc;
    }

    fn and_(&mut self, _can_assign: bool) {
        let l = self.current.line;
        self.emitter()
            .emit_byte(OpCode::JumpIfFalse { jump: 0xFF }, l);
        let end_jump = self.emitter().chunk().code.len() - 1;
        self.emitter().emit_byte(OpCode::Pop, l);
        self.parse_precedence(Precedence::And);
        self.emitter().patch_jump(end_jump);
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

    fn function(&mut self, ftype: FunctionType) {
        let mut scope = ScopeCell::new();
        scope.emitter.function.tpe = ftype;
        self.scope.stack.push(scope);

        self.scope().begin(); 
        
        self.consume(TokenType::LeftParen, "Expect '(' after function name.");


        if !self.check(TokenType::RightParen) {
            loop {
                self.emitter().function.arity += 1;
                // if arity>N error
                let constant = self.parse_variable("Expect parameter name.");
                self.define_variable(constant);
                if !self.matches(TokenType::Comma) { break; }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters.");
        self.consume(TokenType::LeftBrace, "Expect '{' before function body.");
        self.block();

        self.end(self.current.line);

        self.scope.stack.pop();

        let function = self.scope().emitter.function.clone();
        let ftype = ObjType::Function(function);
        let value = Value::Object(ftype);
        let line = self.current.line;
        let index = self.emitter().write_constant(value);
        self.emitter().emit_byte(OpCode::Closure{ index }, line);

    }

    fn fun_declaration(&mut self) {
        let global = self.parse_variable("Expect function name.");
        self.scope().mark_initialized();
        self.function(FunctionType::Function);
        self.define_variable(global);
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.matches(TokenType::Equal) {
            self.expression();
        } else {
            let l = self.current.line;
            self.emitter().emit_byte(OpCode::Nil, l);
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
        let l = self.current.line;
        self.emitter().emit_byte(OpCode::Pop, l);
    }

    fn for_statement(&mut self) {
        self.scope().begin();

        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");
        if self.matches(TokenType::Semicolon) {
            // No inizializer.
        } else if self.matches(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.emitter().chunk().code.len();

        let mut exit_jump = None;

        if !self.matches(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

            // Jump out of the loop if the condition is false.
            let l = self.current.line;
            self.emitter()
                .emit_byte(OpCode::JumpIfFalse { jump: 0xFF }, l);
            exit_jump = Some(self.emitter().chunk().code.len() - 1);

            let l = self.current.line;
            self.emitter().emit_byte(OpCode::Pop, l); // Condition.
        }

        if !self.matches(TokenType::RightParen) {
            let l = self.current.line;
            self.emitter()
                .emit_byte(OpCode::Jump { jump: 0xFF }, l);
            let body_jump = self.emitter().chunk().code.len() - 1;

            let increment_start = self.emitter().chunk().code.len() - 1;
            self.expression();
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

            let l = self.current.line;
            let jump = self.emitter().chunk().code.len() - loop_start;
            self.emitter()
                .emit_byte(OpCode::Loop { jump }, l);

            loop_start = increment_start;

            self.emitter().patch_jump(body_jump);
        }

        self.statement();
        let jump = self.emitter().chunk().code.len() - loop_start;
        let l = self.current.line;
        self.emitter()
            .emit_byte(OpCode::Loop { jump }, l);

        if let Some(jump) = exit_jump {
            let l = self.current.line;        
            self.emitter().patch_jump(jump);
            self.emitter().emit_byte(OpCode::Pop, l); // Condition.
        }

        let l = self.current.line;
        self.scope().end(l); 
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let l = self.current.line;
        self.emitter()
            .emit_byte(OpCode::JumpIfFalse { jump: 0xFF }, l);
        let then_jump = self.emitter().chunk().code.len() - 1;
        self.emitter().emit_byte(OpCode::Pop, l);
        self.statement();

        let l = self.current.line;
        self.emitter()
            .emit_byte(OpCode::Jump { jump: 0xFF }, l);
        let else_jump = self.emitter().chunk().code.len() - 1;

        self.emitter().patch_jump(then_jump);
        self.emitter().emit_byte(OpCode::Pop, l);

        if self.matches(TokenType::Else) {
            self.statement();
        }

        self.emitter().patch_jump(else_jump)
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        let l = self.current.line;
        self.emitter().emit_byte(OpCode::Print, l);
    }

    fn return_statement(&mut self) {
        if self.emitter().function.tpe == FunctionType::Script {
            self.error("Can't return from top-level code.");
        }
        if self.matches(TokenType::Semicolon) {
            let l = self.current.line;
            self.emitter().emit_return(l);
        } else {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after return value.");
            let l = self.current.line;
            self.emitter().emit_byte(OpCode::Return, l);
        }
    }

    fn while_statement(&mut self) {
        let loop_start = self.emitter().chunk().code.len() - 1;
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let l = self.current.line;        
        self.emitter()
            .emit_byte(OpCode::JumpIfFalse { jump: 0xFF }, l);
        let exit_jump = self.emitter().chunk().code.len() - 1;

        let l = self.current.line;
        self.emitter().emit_byte(OpCode::Pop, l);
        
        let l = self.current.line;
        self.statement();

        let jump = self.emitter().chunk().code.len() - loop_start;
        self.emitter()
            .emit_byte(OpCode::Loop { jump }, l);

        self.emitter().patch_jump(exit_jump);
        self.emitter().emit_byte(OpCode::Pop, l);
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
        if self.matches(TokenType::Fun) {
            self.fun_declaration();
        } else if self.matches(TokenType::Var) {
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
        } else if self.matches(TokenType::Return) {
            self.return_statement();
        } else if self.matches(TokenType::While) {
            self.while_statement();
        } else if self.matches(TokenType::LeftBrace) {
            self.scope().begin();
            self.block();
            let l = self.current.line;
            self.scope().end(l);
        } else {
            self.expression_statement();
        }
    }

    pub fn end(&mut self, line: usize) {
        self.emitter().emit_return(line);
        // debug statements
        if !self.had_error {
            self.emitter().chunk().disassemble("code");
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
            TokenType::BangEqual => self.emitter().emit_bytes(OpCode::Equal, OpCode::Not, line),
            TokenType::EqualEqual => self.emitter().emit_byte(OpCode::Equal, line),
            TokenType::Greater => self.emitter().emit_byte(OpCode::Greater, line),
            TokenType::GreaterEqual => self.emitter().emit_bytes(OpCode::Less, OpCode::Not, line),
            TokenType::Less => self.emitter().emit_byte(OpCode::Less, line),
            TokenType::LessEqual => self.emitter().emit_bytes(OpCode::Greater, OpCode::Not, line),
            TokenType::Plus => self.emitter().emit_byte(OpCode::Add, line),
            TokenType::Minus => self.emitter().emit_byte(OpCode::Subtract, line),
            TokenType::Star => self.emitter().emit_byte(OpCode::Multiply, line),
            TokenType::Slash => self.emitter().emit_byte(OpCode::Divide, line),
            _ => {} // Unreachable.
        }
    }

    pub fn call(&mut self, _can_assign: bool) {
        let argc = self.argument_list();
        let l = self.current.line;
        self.emitter().emit_byte(OpCode::Call{ argc }, l);
    }      

    pub fn literal(&mut self, _can_assign: bool) {
        let tok = self.previous.clone();
        match tok.tpe {
            TokenType::False => self.emitter().emit_byte(OpCode::False, tok.line),
            TokenType::Nil => self.emitter().emit_byte(OpCode::Nil, tok.line),
            TokenType::True => self.emitter().emit_byte(OpCode::True, tok.line),
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
