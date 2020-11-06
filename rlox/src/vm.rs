use chunk::Chunk;
use chunk::OpCode;
use compiler;
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
    pub fn interpret(&mut self, source: &String) -> InterpretResult {
        compiler::compile(source.to_string());
        return InterpretResult::Ok;
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
            print!("\n");
            
            self.chunk.disassemble_instruction(self.ip);
            match instruction {
                OpCode::Constant { index } => {
                    let value = self.chunk.read_constant(index);
                    self.stack.push(value);
                }
                OpCode::Add => {
                    let b = self.stack.pop().unwrap().0;
                    let a = self.stack.pop().unwrap().0;
                    self.stack.push(Value(a + b))
                }
                OpCode::Subtract => {
                    let b = self.stack.pop().unwrap().0;
                    let a = self.stack.pop().unwrap().0;
                    self.stack.push(Value(a - b))
                }
                OpCode::Multiply => {
                    let b = self.stack.pop().unwrap().0;
                    let a = self.stack.pop().unwrap().0;
                    self.stack.push(Value(a * b))
                }
                OpCode::Divide   => {
                    let b = self.stack.pop().unwrap().0;
                    let a = self.stack.pop().unwrap().0;
                    self.stack.push(Value(a / b))
                }                
                OpCode::Negate => {
                    let v = self.stack.pop().unwrap();
                    self.stack.push(Value( -v.0 ));
                }
                OpCode::Return => {
                    self.stack.pop().unwrap().print();
                    print!("\n");
                    return InterpretResult::Ok
                }
            }
            self.ip += 1
        }
    }
    
}