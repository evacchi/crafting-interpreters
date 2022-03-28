use value::Value;

#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    Constant { index: usize },
    Nil,
    True,
    False,
    Pop,
    GetLocal { index: usize },
    SetLocal { index: usize },
    GetGlobal { index: usize },
    DefineGlobal { index: usize },
    SetGlobal { index: usize },
    Equal,
    Greater,
    Less,
    Negate,
    Print,
    Jump { jump: usize },
    JumpIfFalse { jump: usize },
    Loop { jump: usize },
    Call { argc: u32 },
    Closure { index: usize },
    Return,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
}

#[derive(Clone,Debug)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    values: Vec<Value>,
    lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            values: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn write_constant(&mut self, value: Value) -> usize {
        let idx = self.values.len();
        self.values.push(value);
        idx
    }

    pub fn read_constant(&self, offset: usize) -> Value {
        self.values[offset].clone()
    }

    pub fn write(&mut self, op: OpCode, line: usize) {
        self.code.push(op);
        self.lines.push(line);
    }

    pub fn fetch(&self, ip: usize) -> OpCode {
        self.code[ip]
    }

    pub fn line_at(&self, ip: usize) -> usize {
        self.lines[ip]
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        for i in 0..self.code.len() {
            self.disassemble_instruction(i);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) {
        print!("{:04} ", offset);
        let op = &self.code[offset];
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ")
        } else {
            print!("{:4} ", self.lines[offset]);
        }
        match op {
            OpCode::Constant { index } => self.constant_instruction("CONSTANT", index),
            OpCode::Nil => self.simple_instruction("NIL"),
            OpCode::True => self.simple_instruction("TRUE"),
            OpCode::False => self.simple_instruction("FALSE"),
            OpCode::Pop => self.simple_instruction("POP"),
            OpCode::GetLocal { index } => self.byte_instruction("GET_LOCAL", index),
            OpCode::SetLocal { index } => self.byte_instruction("SET_LOCAL", index),
            OpCode::GetGlobal { index } => self.constant_instruction("GET_GLOBAL", index),
            OpCode::DefineGlobal { index } => self.constant_instruction("DEFINE_GLOBAL", index),
            OpCode::SetGlobal { index } => self.constant_instruction("SET_GLOBAL", index),
            OpCode::Equal => self.simple_instruction("EQUAL"),
            OpCode::Greater => self.simple_instruction("GREATER"),
            OpCode::Less => self.simple_instruction("LESS"),
            OpCode::Add => self.simple_instruction("ADD"),
            OpCode::Subtract => self.simple_instruction("SUBTRACT"),
            OpCode::Multiply => self.simple_instruction("MULTIPLY"),
            OpCode::Divide => self.simple_instruction("DIVIDE"),
            OpCode::Not => self.simple_instruction("NOT"),
            OpCode::Negate => self.simple_instruction("NEGATE"),
            OpCode::Print => self.simple_instruction("PRINT"),
            OpCode::JumpIfFalse { jump } => {
                self.jump_instruction("OP_JUMP_IF_FALSE", 1, offset + 1, jump)
            }
            OpCode::Jump { jump } => self.jump_instruction("OP_JUMP", 1, offset + 1, jump),
            OpCode::Loop { jump } => self.jump_instruction("OP_LOOP", -1, offset + 1, jump),
            OpCode::Call { .. } => self.simple_instruction("OP_CALL"),
            OpCode::Closure { index } => self.constant_instruction("OP_CLOSURE", index),
            OpCode::Return => self.simple_instruction("RETURN"),
        }
    }

    fn constant_instruction(&self, op: &str, offset: &usize) {
        print!("{:16} {:4} '", op, offset);
        self.values[*offset].print();
        println!("'");
    }

    fn simple_instruction(&self, op: &str) {
        println!("{}", op)
    }

    fn byte_instruction(&self, op: &str, offset: &usize) {
        println!("{:16} {:4}", op, offset)
    }

    fn jump_instruction(&self, op: &str, sign: isize, offset: usize, jump: &usize) {
        if sign < 0 {
            println!("{:16} {:4}", op, offset - jump);
        } else {
            println!("{:16} {:4}", op, offset + jump);
        }
    }
}
