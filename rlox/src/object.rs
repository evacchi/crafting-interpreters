use chunk::Chunk;
use value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionType {
    Script, Function
}


#[derive(Debug, Clone)]
pub struct Closure {
    pub function: Function,
    pub upvalues: Vec<Upvalue>,
}

impl Closure {
    pub fn new(function: Function) -> Closure {
        Closure {
            function,
            upvalues: Vec::new()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub arity: u32,
    pub chunk: Chunk,
    pub name: Option<String>,
    pub tpe: FunctionType
}

impl Function {
    pub fn named(name: String, arity: u32) -> Function {
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

#[derive(Clone)]
pub struct Native {
    pub name: String,
    pub arity: u32,
    pub fun: fn (&[Value]) -> Value
}

impl Native {
    pub fn named(name: String, arity: u32, fun: fn (&[Value]) -> Value) -> Native {
        Native { name, arity, fun }
    }
}

impl std::fmt::Debug for Native {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn {}/{}", self.name, self.arity)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Upvalue {
    pub index: usize,
    pub is_local: bool
}

#[derive(Debug, Clone)]
pub enum ObjType {
    String(String),
    Function(Function),
    Closure(Closure),
    NativeFn(Native),
    Upvalue(Upvalue)
}

impl PartialEq for ObjType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ObjType::Upvalue(a), ObjType::Upvalue(b)) => 
                a == b,
            (ObjType::String(a), ObjType::String(b)) => 
                a == b,
            (ObjType::Function(Function{ arity: arity1, name: name1 , tpe: tpe1, ..}), 
             ObjType::Function(Function{ arity: arity2, name: name2 , tpe: tpe2, ..})) =>
                arity1 == arity2 && name1 == name2 && tpe1 == tpe2,
            (ObjType::Closure(Closure{ function: f1, upvalues: upvs1}), 
             ObjType::Closure(Closure{ function: f2, upvalues: upvs2})) =>
                f1.arity == f2.arity && f1.name == f2.name && f1.tpe == f2.tpe && upvs1 == upvs2,
            _ => false
        }
    }
}
