use std::rc::Rc;

#[derive(Debug,Clone,PartialEq)]
pub enum ObjType {
    String(Rc<String>)
}