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
                OpCode::OpConstant { value, line } => {
                    self.stack.push(value);
                    print!("\n")
                }
                OpCode::OpReturn { line } => {
                    self.stack.pop().unwrap().print();
                    print!("\n");
                    return InterpretResult::Ok
                }
            }
            self.ip += 1
        }
    }
}