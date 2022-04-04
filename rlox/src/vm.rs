use chunk::Chunk;
use chunk::OpCode;
use compiler::Parser;
use compiler::Compiler;
use memory::Memory;
use object::Function;
use object::ObjType;
use object::Native;
use object::Closure;
use object::Upvalue;
use value::Value;

#[derive(Clone)]
pub struct CallFrame {
    closure: Closure, // ptr would be better, but let's use a clone for now
    ip: usize,
    slot: usize
}

impl CallFrame {
    fn new(closure: Closure, slot: usize) -> CallFrame {
        CallFrame {
            closure,
            ip: 0,
            slot
        }
    }
}

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    open_upvalues: Vec<Upvalue>,
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
            frames: Vec::new(),
            stack: Vec::new(),
            open_upvalues: Vec::new(),
            memory: Memory::new(),
        }
    }

    fn native_clock(_args: &[Value]) -> Value {
        let t = std::time::SystemTime::now();
        let elapsed = t.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as f64;
        Value::Number(elapsed)
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let parser = &mut Parser::new(source.to_string());
        let mut compiler = Compiler::new(parser);
        if let Some(function) = compiler.compile() {
            self.stack.push(Value::Object(ObjType::Function(function.clone())));
            let frame = CallFrame::new(Closure::new(function.clone()), 0);
            let emitter = compiler.state();
            self.memory = emitter.memory;
            // FIXME ensure native functions are defined because memory is being overwrittend
            self.define_native(Native::named("clock".to_string(), 0, VM::native_clock));
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
            let instruction = frame.closure.function.chunk.fetch(frame.ip);

            print!("          ");
            for slot in self.stack.iter() {
                print!("[ ");
                slot.print();
                print!(" ]");
            }
            println!();

            frame.closure.function.chunk.disassemble_instruction(frame.ip);

            frame.ip += 1;

            match instruction {
                OpCode::Constant { index } => {
                    let value = frame.closure.function.chunk.read_constant(index);
                    self.stack.push(value);
                }
                OpCode::Nil => self.stack.push(Value::Nil),
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::False => self.stack.push(Value::Bool(false)),
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::GetLocal { index } => {
                    // self.stack.iter().for_each(|e| println!("STACK: {:?}", e));
                    self.stack.push(self.stack[frame.slot + index].clone());
                }
                OpCode::SetLocal { index } => {
                    self.stack[frame.slot + index-1] = self.stack.last().unwrap().clone()
                }
                OpCode::GetGlobal { index } => {
                    let value = frame.closure.function.chunk.read_constant(index);

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
                    let value = frame.closure.function.chunk.read_constant(index);

                    if let Value::Object(ObjType::String(s)) = value {
                        self.memory
                            .set_global(s.to_string(), self.stack.last().unwrap().clone());
                        self.stack.pop();
                    }
                }
                OpCode::SetGlobal { index } => {
                    let value = frame.closure.function.chunk.read_constant(index);

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
                OpCode::GetUpvalue { index } => {
                    println!("GET UP {:?}", frame.closure.upvalues);
                    let idx = frame.slot + frame.closure.upvalues[index].index;
                    self.stack.push(self.stack[idx].clone());
                }
                OpCode::SetUpvalue { index } => {
                    if let Some(value) = self.stack.last() {
                        let idx = frame.slot + frame.closure.upvalues[index].index;
                        self.stack[idx] = value.clone();
                    } else {
                        panic!("Error SetUpValue: Cannot find value at index {}", index);
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
                OpCode::Call { argc } => {
                    let up = self.stack.len() - 1;
                    let a = argc as usize;
                    let offset = up - a;
                    let args = &self.stack[offset..up+1];
                    let callee = args[0].clone();

                    match callee {
                        Value::Object(ObjType::Function(f)) => 
                            if argc == f.arity {
                                self.frames.push(CallFrame::new(Closure::new(f.clone()), up));                          
                            } else {
                                self.runtime_error(& format!("Expected {} arguments but got {}.", f.arity, argc));
                            }
                        Value::Object(ObjType::NativeFn(f)) =>
                            if argc == f.arity {
                                let result = (f.fun)(args);
                                self.stack.push(result);                 
                            } else {
                                self.runtime_error(& format!("Expected {} arguments but got {}.", f.arity, argc));
                            }
                        Value::Object(ObjType::Closure(f)) => 
                            if argc == f.function.arity {
                                let cl = Closure::new(f.function.clone());
                                self.frames.push(CallFrame::new(cl, up));                          
                            } else {
                                self.runtime_error(& format!("Expected {} arguments but got {}.", f.function.arity, argc));
                            }
                        w => {
                            println!("GOT {:?}", w);

                            self.runtime_error("Can only call functions and classes");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Closure { index, upvalues } => {
                    println!("CLOSURE {:?}", upvalues);
                    let fc = frame.closure.function.chunk.read_constant(index);
                    if let Value::Object(ObjType::Function(function)) = fc {
                        let us: Vec<Upvalue> = upvalues.iter()
                            .map(|u|
                                if u.is_local {
                                    Upvalue{index: frame.slot + u.index, is_local: u.is_local}
                                } else {
                                    u.clone()
                                })
                            .collect();
                        let closure = Closure{function, upvalues: us};
                        self.stack.push(Value::Object(ObjType::Closure(closure)));
                    } else {
                        panic!("I was expecting a function.");
                    }
                }
                OpCode::CloseUpvalue => {
                    //         closeUpvalues(vm.stackTop - 1);
                    //        pop();
                    self.close_upvalues(self.stack.len());
                    self.stack.pop();
                }
                OpCode::Return => {
                    if let Some(result) = self.stack.pop() {

                        // closeUpvalues(frame->slots)
                        let slot = frame.slot;
                        self.close_upvalues(slot);
                        self.frames.pop();
                        for i in slot..self.stack.len() {
                            println!("{}/{}", i, slot);
                            self.stack.pop();
                        }
                        if self.frames.len() == 0 {
                            self.stack.pop();
                            // Exit interpreter.
                            return InterpretResult::Ok;
                        }
                        self.stack.push(result);        
                    }
                }
            }
        }
    }

    fn close_upvalues(&mut self, index: usize) {
        let mut iter = self.open_upvalues.len();
        println!("OPEN: {:?}", self.open_upvalues);
        loop {
            if iter == 0 || self.open_upvalues.last().unwrap().index < index {
                break;
            } 

            self.open_upvalues.pop();
            
            iter -= 1;
        }
    }

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{}", message);
        // let line = self.chunk.line_at(self.ip);
        let line = -1;
        eprint!("[line {}] in script\n", line);

        self.stack.clear();
    }

    fn define_native(&mut self, fun: Native) {
        self.memory.set_global(fun.name.to_string(), Value::Object(ObjType::NativeFn(fun)));
    }
}
