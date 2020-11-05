mod chunk;
mod value;

use chunk::Chunk;
use chunk::OpCode;


fn main() {
    let mut chunk = Chunk::new();
    
    chunk.write(OpCode::OpReturn { line: 123 });
    chunk.write(OpCode:: OpConstant{ value: 1.2, line: 123});

    chunk.disassemble("test chunk");
}
