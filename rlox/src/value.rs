use std::rc::Rc;
use object::ObjType;

#[derive(Debug,Clone,PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Object(Rc<ObjType>)
}

impl Value {
    pub fn print(&self) {
        print!("{:?}", self);
    }
}