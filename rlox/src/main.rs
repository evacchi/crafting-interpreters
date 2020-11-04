type Value = f64;

#[derive(Debug)]
enum OpCode {
    OpConstant { value: Value, line: usize },
    OpReturn   { line: usize },
}

struct Chunk {
    code: Vec<OpCode>,
}

impl Chunk {

    fn write(&mut self, op: OpCode) {
        self.code.push(op)
    }

    fn disassemble(&self, name: &str) {
        print!("== {} ==\n", name);
    
        for i in 0..self.code.len() {
            self.disassemble_instruction(i);
        }
    }

    fn disassemble_instruction(&self, offset: usize) {
        print!("{:04} {:?}\n", offset, &self.code[offset]);
    }
    

}

fn main() {
    let mut chunk = Chunk {
        code : Vec::new(),
    };
    chunk.write(OpCode::OpReturn { line: 123 });
    chunk.write(OpCode:: OpConstant{ value: 1.2, line: 123});

    chunk.disassemble("test chunk");
}
