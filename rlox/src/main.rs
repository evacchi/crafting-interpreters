extern crate core;

mod chunk;
mod compiler;
mod memory;
mod object;
mod scanner;
mod value;
mod vm;

use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process;

use vm::InterpretResult;
use vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut vm = VM::new();

    if args.len() == 1 {
        repl(&mut vm);
    } else if args.len() == 2 {
        run_file(&mut vm, &args[1]);
    } else {
        eprint!("Usage: clox [path]\n");
        process::exit(64);
    }
}

fn repl(vm: &mut VM) {
    loop {
        let mut line = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut line)
            .expect("error: unable to read user input");
        if line.is_empty() {
            println!();
            break;
        }
        vm.interpret(&line);
    }
}

fn run_file(vm: &mut VM, f: &str) {
    let source = fs::read_to_string(f).expect("Could not open file");
    match vm.interpret(&source) {
        InterpretResult::Ok => {}
        InterpretResult::CompileError => {
            process::exit(65);
        }
        InterpretResult::RuntimeError => {
            process::exit(70);
        }
    }
}
