use value::Value;

#[derive(Debug,Copy,Clone)]
pub enum OpCode {
    Constant { offset: usize, line: usize },
    Negate   { line: usize },
    Return   { line: usize },
    Add      { line: usize },
    Subtract { line: usize },
    Multiply { line: usize },
    Divide   { line: usize },
}

impl OpCode {
    pub fn disassemble(&self, offset: usize) {
        print!("{:04} {:?}\n", offset, &self);
    }
}

pub struct Chunk {
    code: Vec<OpCode>,
    values: Vec<Value>
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code : Vec::new(),
            values : Vec::new(),
        }
    }

    pub fn write_constant(&mut self, value: Value) -> usize {
        let idx = self.values.len();
        self.values.push(value);
        idx
    }

    pub fn read_constant(&self, offset: usize) -> Value {
        self.values[offset]
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