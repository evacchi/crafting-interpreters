use chunk::Chunk;

#[derive(Debug, Clone)]
pub enum ObjType {
    String(String),
    Function(i32,Chunk,Option<String>)
}

impl PartialEq for ObjType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ObjType::String(a), ObjType::String(b)) => 
                a == b,
            (ObjType::Function(arity1,_,name1), ObjType::Function(arity2,_,name2)) =>
                arity1 == arity2 && name1 == name2,
            _ => false
        }
    }
}
