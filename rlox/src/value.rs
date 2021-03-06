use object::ObjType;

#[derive(Debug,Clone,PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Object(ObjType)
}

impl Value {
    pub fn print(&self) {
        let s = match self {
            Value::Nil => String::from("nil"),
            Value::Bool(b) => format!("{}", b),
            Value::Number(n) => format!("{}", n),
            Value::Object(ObjType::String(s)) => format!("{:?}", s),
        };
        print!("{}", s);
    }
}