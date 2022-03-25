use chunk::Chunk;

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionType {
    Script, Function
}

#[derive(Debug, Clone)]
pub struct Function {
    pub arity: i32,
    pub chunk: Chunk,
    pub name: Option<String>,
    pub tpe: FunctionType
}

impl Function {
    pub fn named(arity: i32, name: String) -> Function {
        Function {
            arity,
            chunk: Chunk::new(),
            name: Some(name),
            tpe: FunctionType::Function
        }
    }
    pub fn main() -> Function {
        Function {
            arity: 0,
            chunk: Chunk::new(),
            name: None,
            tpe: FunctionType::Script
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
            (ObjType::Function(Function{ arity: arity1, name: name1 , tpe: tpe1, ..}), 
             ObjType::Function(Function{ arity: arity2, name: name2 , tpe: tpe2, ..})) =>
                arity1 == arity2 && name1 == name2 && tpe1 == tpe2,
            _ => false
        }
    }
}
