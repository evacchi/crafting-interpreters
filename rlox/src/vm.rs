use chunk::Chunk;
use chunk::OpCode;
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
    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.chunk = chunk;
        self.ip = 0;
        return self.run();
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
            
            instruction.disassemble(self.ip);
            match instruction {
                OpCode::Constant { offset, line } => {
                    let value = self.chunk.read_constant(offset);
                    self.stack.push(value);
                }
                OpCode::Add      { line } => {
                    let b = self.stack.pop().unwrap().0;
                    let a = self.stack.pop().unwrap().0;
                    self.stack.push(Value(a + b))
                }
                OpCode::Subtract { line } => {
                    let b = self.stack.pop().unwrap().0;
                    let a = self.stack.pop().unwrap().0;
                    self.stack.push(Value(a - b))
                }
                OpCode::Multiply { line } => {
                    let b = self.stack.pop().unwrap().0;
                    let a = self.stack.pop().unwrap().0;
                    self.stack.push(Value(a * b))
                }
                OpCode::Divide   { line } => {
                    let b = self.stack.pop().unwrap().0;
                    let a = self.stack.pop().unwrap().0;
                    self.stack.push(Value(a / b))
                }                
                OpCode::Negate { line } => {
                    let v = self.stack.pop().unwrap();
                    self.stack.push(Value( -v.0 ));
                }
                OpCode::Return { line } => {
                    self.stack.pop().unwrap().print();
                    print!("\n");
                    return InterpretResult::Ok
                }
            }
            self.ip += 1
        }
    }
    
}