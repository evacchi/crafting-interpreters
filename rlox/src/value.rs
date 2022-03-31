use object::ObjType;
use object::Function;
use object::Native;
use object::Closure;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Object(ObjType),
}

impl Value {
    pub fn print(&self) {
        let s = match self {
            Value::Nil => String::from("nil"),
            Value::Bool(b) => format!("{}", b),
            Value::Number(n) => format!("{}", n),
            Value::Object(ObjType::String(s)) => format!("{:?}", s),
            Value::Object(ObjType::Upvalue(s)) => format!("{:?}", s),
            Value::Object(ObjType::Function(Function{ arity, name, .. })) => 
                match name {
                    Some(name) => format!("<fn {:?}/{:?}>", name, arity),
                    None => format!("<script>"),
                },
            Value::Object(ObjType::Closure(Closure{ function, .. })) => 
                match function.name.clone() {
                    Some(name) => format!("<closure fn {:?}/{:?}>", name, function.arity),
                    None => format!("<script>"),
                },

            Value::Object(ObjType::NativeFn( Native { arity, name, .. } )) => 
                format!("<native fn {:?}/{:?}>", name, arity),
        };
        print!("{}", s);
    }
}
