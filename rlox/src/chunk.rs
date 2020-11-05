use value::Value;

#[derive(Debug)]
pub enum OpCode {
    OpConstant { value: Value, line: usize },
    OpReturn   { line: usize },
}

pub struct Chunk {
    code: Vec<OpCode>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code : Vec::new(),
        }
    }

    pub fn write(&mut self, op: OpCode) {
        self.code.push(op)
    }

    pub fn disassemble(&self, name: &str) {
        print!("== {} ==\n", name);
    
        for i in 0..self.code.len() {
            self.disassemble_instruction(i);
        }
    }

    fn disassemble_instruction(&self, offset: usize) {
        print!("{:04} {:?}\n", offset, &self.code[offset]);
    }
}