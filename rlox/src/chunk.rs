use value::Value;

#[derive(Debug,Copy,Clone)]
pub enum OpCode {
    Constant { value: Value, line: usize },
    Negate   { line: usize },
    Return   { line: usize },
}

impl OpCode {
    pub fn disassemble(&self, offset: usize) {
        print!("{:04} {:?}\n", offset, &self);
    }
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

    pub fn fetch(&self, ip: usize) -> OpCode {
        self.code[ip]
    } 

    pub fn disassemble(&self, name: &str) {
        print!("== {} ==\n", name);
    
        for (i, op) in self.code.iter().enumerate() {
            op.disassemble(i);
        }
    }

}