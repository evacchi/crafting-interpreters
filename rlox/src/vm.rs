use chunk::Chunk;
use chunk::OpCode;

pub struct VM {
    chunk: Chunk,
    ip: usize
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError
}

impl VM {
    pub fn new() -> VM {
        VM { chunk: Chunk::new(), ip: 0 }
    }
    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.chunk = chunk;
        self.ip = 0;
        return self.run();
    }
    fn run(&mut self) -> InterpretResult {
        loop {
            let instruction = self.chunk.fetch(self.ip);
            instruction.disassemble(self.ip);
            match instruction {
                OpCode::OpConstant { value, line } => {
                    value.print();
                    print!("\n")
                }
                OpCode::OpReturn { line } => 
                    return InterpretResult::Ok,
            }
            self.ip += 1
        }
    }
}