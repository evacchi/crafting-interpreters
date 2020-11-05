mod chunk;
mod vm;
mod value;

use chunk::Chunk;
use chunk::OpCode;

use value::Value;

use vm::VM;

fn main() {
    let mut vm = VM::new();
    let mut chunk = Chunk::new();

    chunk.write(OpCode::OpReturn { line: 123 });
    chunk.write(OpCode:: OpConstant{ value: Value(1.2), line: 123});

    chunk.disassemble("test chunk");
    vm.interpret(chunk);
}
