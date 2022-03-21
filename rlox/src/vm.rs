use chunk::Chunk;
use chunk::OpCode;
use compiler::Compiler;
use memory::Memory;
use object::Function;
use object::ObjType;
use value::Value;

#[derive(Clone)]
pub struct CallFrame {
    function: Function, // ptr would be better, but let's use a clone for now
    ip: usize,
    slots: Vec<Value>
}

impl CallFrame {
    fn new(function: Function, slots: Vec<Value>) -> CallFrame {
        CallFrame {
            function: function,
            ip: 0,
            slots: slots//??
        }
    }
}

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    memory: Memory,
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl VM {
    pub fn new() -> VM {
        VM {
            chunk: Chunk::new(),
            ip: 0,
            stack: Vec::new(),
            frames: Vec::new(),
            memory: Memory::new(),
        }
    }
    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut compiler = Compiler::new(source.to_string());
        if let Some(function) = compiler.compile() {
            let frame = CallFrame::new(function.clone(), self.stack.clone());
            let (chk, mem) = compiler.state();
            self.chunk = chk;
            self.memory = mem;
            self.ip = 0;
            self.frames.push(frame);
            self.run()
        } else {
            return InterpretResult::CompileError;
        }

    }

    fn is_falsey(value: Value) -> bool {
        match value {
            Value::Nil => true,
            Value::Bool(b) => !b,
            _ => false,
        }
    }

    fn binary_op(&mut self, op: fn(f64, f64) -> f64) {
        if let (&Value::Number(b), &Value::Number(a)) = (
            self.stack.last().unwrap(),
            self.stack.get(self.stack.len() - 2).unwrap(),
        ) {
            self.stack.pop();
            self.stack.pop();
            self.stack.push(Value::Number(op(a, b)))
        }
    }

    fn bool_op(&mut self, op: fn(f64, f64) -> bool) {
        let bb = self.stack.last().unwrap();
        let aa = self.stack.get(self.stack.len() - 2).unwrap();
        if let (&Value::Number(b), &Value::Number(a)) = (bb, aa) {
            self.stack.pop();
            self.stack.pop();
            self.stack.push(Value::Bool(op(a, b)))
        }
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            let frame = self.frames.last_mut().unwrap();
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
                OpCode::Nil => self.stack.push(Value::Nil),
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::False => self.stack.push(Value::Bool(false)),
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::GetLocal { index } => self.stack.push(frame.slots[index].clone()),
                OpCode::SetLocal { index } => {
                    frame.slots[index] = self.stack.last().unwrap().clone()
                }
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
                            Some(v) => self.stack.push(v.clone()),
                        }
                    }
                }
                OpCode::DefineGlobal { index } => {
                    let value = self.chunk.read_constant(index);

                    if let Value::Object(ObjType::String(s)) = value {
                        self.memory
                            .set_global(s.to_string(), self.stack.last().unwrap().clone());
                        self.stack.pop();
                    }
                }
                OpCode::SetGlobal { index } => {
                    let value = self.chunk.read_constant(index);

                    if let Value::Object(ObjType::String(s)) = value {
                        if self
                            .memory
                            .set_global(s.to_string(), self.stack.last().unwrap().clone())
                        {
                            self.memory.delete_global(s.to_string());
                            self.runtime_error(&format!("Undefined variable '{}'.", s));
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(a == b))
                }
                OpCode::Greater => self.bool_op(|a, b| a > b),
                OpCode::Less => self.bool_op(|a, b| a < b),
                OpCode::Add => {
                    match (self.stack.last().unwrap().clone(),
                            self.stack.get(self.stack.len() - 2).unwrap().clone()) {
                        (Value::Object(bref), Value::Object(aref)) => match (aref, bref) {
                            (ObjType::String(a), ObjType::String(b)) => {
                                self.stack.pop();
                                self.stack.pop();

                                let owned = format!("{}{}", a, b);

                                self.stack.push(Value::Object(ObjType::String(owned)));
                            }
                            _ => self.runtime_error("Cannot concatenate")
                        },
                        (Value::Number(b), Value::Number(a)) => {
                            self.stack.pop();
                            self.stack.pop();
                            self.stack.push(Value::Number(a + b))
                        }
                        _ => {
                            self.runtime_error("Operands must be two numbers or two strings.");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Subtract => self.binary_op(|a, b| a - b),
                OpCode::Multiply => self.binary_op(|a, b| a * b),
                OpCode::Divide => self.binary_op(|a, b| a / b),
                OpCode::Not => {
                    let v = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(VM::is_falsey(v)))
                }
                OpCode::Negate => {
                    if let Value::Number(n) = self.stack.pop().unwrap() {
                        self.stack.push(Value::Number(-n));
                    } else {
                        self.runtime_error("Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Print => {
                    self.stack.pop().unwrap().print();
                    println!();
                }
                OpCode::JumpIfFalse { jump } => {
                    let x = self.stack.last().unwrap().clone();
                    if VM::is_falsey(x) {
                        frame.ip += jump;
                    }
                }
                OpCode::Jump { jump } => {
                    frame.ip += jump;
                }
                OpCode::Loop { jump } => {
                    frame.ip -= jump;
                }
                OpCode::Return => {
                    // Exit interpreter.
                    return InterpretResult::Ok;
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
