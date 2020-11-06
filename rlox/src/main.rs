mod chunk;
mod compiler;
mod scanner;
mod vm;
mod value;

use std::env;
use std::fs;
use std::io;
use std::process;

use chunk::Chunk;
use chunk::OpCode;

use value::Value;

use vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut vm = VM::new();

    if args.len() == 1 {
        repl(&mut vm);
    } else if args.len() == 2 {
        runFile(&mut vm, &args[1]);
    } else {
        eprint!("Usage: clox [path]\n");
        process::exit(64);
    }
}

fn repl(vm: &mut VM) {
    let mut line = String::new();
    loop {
        print!("> ");
        io::stdin().read_line(&mut line)
            .expect("error: unable to read user input");
            if (line.is_empty()) {
                println!();
                break;
            }
        vm.interpret(&line);
    }
}

fn runFile(vm: &mut VM, f: &String) {
    let source = fs::read_to_string(f)
                    .expect("Could not open file");
    let result = vm.interpret(&source);
  
    // if (result == INTERPRET_COMPILE_ERROR) process::exit(65);
    // if (result == INTERPRET_RUNTIME_ERROR) process::exit(70);
}
