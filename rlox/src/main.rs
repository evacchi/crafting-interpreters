#[derive(Debug)]
enum OpCode {
    OpReturn,
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

    fn disassemble_instruction(&self, offset: usize) -> usize{
        print!("{:04} ", offset);
    
        let instruction = &self.code[offset];
        match instruction {
          OpCode::OpReturn =>
            return simple_instruction("OpReturn", offset),
        }
    }
    

}



fn simple_instruction(name: &str, offset: usize) -> usize {
    print!("{}\n", name);
    return offset + 1;
}

fn main() {
    let mut chunk = Chunk {
        code : Vec::new()
    };
    chunk.write(OpCode::OpReturn);
    chunk.disassemble("test chunk");
}
