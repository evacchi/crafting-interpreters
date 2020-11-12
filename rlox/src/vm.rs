use chunk::Chunk;
use chunk::OpCode;
use compiler::Compiler;
use object::ObjType;
use value::Value;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError
}

impl VM {
    pub fn new() -> VM {
        VM { chunk: Chunk::new(), ip: 0, stack: Vec::new() }
    }
    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut compiler = Compiler::new(source.to_string());
        if !compiler.compile() {
            return InterpretResult::CompileError;
        }
        self.chunk = compiler.chunk();
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
                OpCode::Equal    => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(a == b))
                }
                OpCode::Greater  => self.bool_op(|a, b| a > b),
                OpCode::Less     => self.bool_op(|a, b| a < b),
                OpCode::Add      =>{
                    match (self.stack.last().unwrap().clone(), self.stack.get(self.stack.len()-2).unwrap().clone()) {
                        (Value::Object(ObjType::String(a)), Value::Object(ObjType::String(b))) => {
                            self.stack.pop();
                            self.stack.pop();

                            let owned = format!("{}{}", a, b);
                            
                            self.stack.push(Value::Object(ObjType::String(owned)));
                            
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
                OpCode::Return => {
                    self.stack.pop().unwrap().print();
                    println!();
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