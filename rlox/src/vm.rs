use std::rc::Rc;

use chunk::Chunk;
use chunk::OpCode;
use compiler::Compiler;
use memory::Memory;
use object::ObjType;
use value::Value;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    memory: Memory,
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError
}

impl VM {
    pub fn new() -> VM {
        VM { chunk: Chunk::new(), ip: 0, stack: Vec::new(), memory: Memory::new() }
    }
    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut compiler = Compiler::new(source.to_string());
        if !compiler.compile() {
            return InterpretResult::CompileError;
        }
        let (chk, mem) = compiler.state();
        self.chunk = chk;
        self.memory = mem;
        self.ip = 0;
        self.run()
    }

    fn is_falsey(&self, value: Value) -> bool {
        match value {
            Value::Nil => true,
            Value::Bool(b) => b,
            _ => false
        }
    }

    fn binary_op(&mut self, op: fn(f64,f64) -> f64) {
        if let (&Value::Number(b), &Value::Number(a)) =  (self.stack.last().unwrap(), self.stack.get(self.stack.len()-2).unwrap()) {
            self.stack.pop();
            self.stack.pop();
            self.stack.push(Value::Number(op(a,b)))
        }
    }

    fn bool_op(&mut self, op: fn(f64,f64) -> bool) {
        if let (&Value::Number(b), &Value::Number(a)) =  (self.stack.last().unwrap(), self.stack.get(self.stack.len()-2).unwrap()) {
            self.stack.pop();
            self.stack.pop();
            self.stack.push(Value::Bool(op(a,b)))
        }
    }


    fn run(&mut self) -> InterpretResult {
        loop {
            let instruction = self.chunk.fetch(self.ip);

            print!("          ");
            for slot in self.stack.iter() {
              print!("[ ");
              slot.print();
              print!(" ]");
            }
            println!();
            
            self.chunk.disassemble_instruction(self.ip);
            match instruction {
                OpCode::Constant { index } => {
                    let value = self.chunk.read_constant(index);
                    self.stack.push(value);
                }
                OpCode::Nil      => self.stack.push(Value::Nil),
                OpCode::True     => self.stack.push(Value::Bool(true)),
                OpCode::False    => self.stack.push(Value::Bool(false)),
                OpCode::Pop      => { self.stack.pop(); }
                OpCode::GetGlobal { index } => {
                    let value = self.chunk.read_constant(index);

                    if let Value::Object(ObjType::String(s)) = value {
                        let k = s.to_string();
                        match self.memory.get_global(k) {
                            None => {
                                let ss = format!("Undefined variable '{}'.", s);
                                self.runtime_error(&ss);
                                return InterpretResult::RuntimeError;
                            }
                            Some(v) => 
                                self.stack.push(v.clone())
                        }
                    }
                }
                OpCode::DefineGlobal { index } => {
                    let value = self.chunk.read_constant(index);

                    if let Value::Object(ObjType::String(s)) = value {
                        self.memory.set_global(s.to_string(), self.stack.last().unwrap().clone());
                        self.stack.pop();
                    }
                }
                OpCode::SetGlobal { index } => {
                    let value = self.chunk.read_constant(index);

                    if let Value::Object(ObjType::String(s)) = value {
                        if self.memory.set_global(s.to_string(), self.stack.last().unwrap().clone()) {
                            self.memory.delete_global(s.to_string());
                            let ss = format!("Undefined variable '{}'.", s);
                            self.runtime_error(&ss);
                            return InterpretResult::RuntimeError;
                        }
                        self.stack.pop();
                    }
                }
                OpCode::Equal    => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(a == b))
                }
                OpCode::Greater  => self.bool_op(|a, b| a > b),
                OpCode::Less     => self.bool_op(|a, b| a < b),
                OpCode::Add      =>{
                    match (self.stack.last().unwrap().clone(), self.stack.get(self.stack.len()-2).unwrap().clone()) {
                        (Value::Object(aref), Value::Object(bref)) => {
                            match (aref, bref) {
                                (ObjType::String(a), ObjType::String(b)) => {

                                    self.stack.pop();
                                    self.stack.pop();
        
                                    let owned = format!("{}{}", a, b);
                                    
                                    self.stack.push(Value::Object(ObjType::String(Rc::new(owned))));
                                }
                            }
                        }
                        (Value::Number(b), Value::Number(a)) => {
                            self.stack.pop();
                            self.stack.pop();
                            self.stack.push(Value::Number(a + b))
                        }
                        _ => {
                            self.runtime_error(
                                "Operands must be two numbers or two strings.");
                            return InterpretResult::RuntimeError;
                        }

                    }
                }
                OpCode::Subtract => self.binary_op(|a, b| a - b),
                OpCode::Multiply => self.binary_op(|a, b| a * b),
                OpCode::Divide   => self.binary_op(|a, b| a / b),
                OpCode::Not      => {
                    let v = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(self.is_falsey(v)))
                }
                OpCode::Negate   => {
                    if let Value::Number(n) = self.stack.pop().unwrap() {
                        self.stack.push(Value::Number( -n ));                        
                    } else {
                        self.runtime_error("Operand must be a number.");
                        return InterpretResult::RuntimeError
                    }
                }
                OpCode::Print => {
                    self.stack.pop().unwrap().print();
                    println!();
                }
                OpCode::Return => {
                    // Exit interpreter.
                    return InterpretResult::Ok
                }
            }
            self.ip += 1
        }
    }

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{}", message);
        let line = self.chunk.line_at(self.ip);
        eprint!("[line {}] in script\n", line);
      
        self.stack.clear();
      }
      
    
}