use chunk::Chunk;

#[derive(Debug, Clone)]
pub struct Function {
    pub arity: i32,
    pub chunk: Chunk,
    pub name: Option<String>
}

impl Function {
    pub fn named(arity: i32, name: String) -> Function {
        Function {
            arity,
            chunk: Chunk::new(),
            name: Some(name)
        }
    }
    pub fn main() -> Function {
        Function {
            arity: 0,
            chunk: Chunk::new(),
            name: None
        }
    }
}

#[derive(Debug, Clone)]
pub enum ObjType {
    String(String),
    Function(Function)
}

impl PartialEq for ObjType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ObjType::String(a), ObjType::String(b)) => 
                a == b,
            (ObjType::Function(Function{ arity: arity1, chunk: _, name: name1 }), 
             ObjType::Function(Function{ arity: arity2, chunk: _, name: name2 })) =>
                arity1 == arity2 && name1 == name2,
            _ => false
        }
    }
}
