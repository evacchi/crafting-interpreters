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

    let index = chunk.write_constant(Value(1.2));
    chunk.write(OpCode::Constant{ index }, 123);

    let index = chunk.write_constant(Value(3.4));
    chunk.write(OpCode::Constant{ index }, 123);
    chunk.write(OpCode::Add, 123);

    let index = chunk.write_constant(Value(5.6));
    chunk.write(OpCode::Constant{ index }, 123);

    chunk.write(OpCode::Divide, 123);
    chunk.write(OpCode::Negate, 123);
    chunk.write(OpCode::Return, 123);
    
    chunk.disassemble("test chunk");
    vm.interpret(chunk);
}
