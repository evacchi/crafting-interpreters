use std::rc::Rc;

use object::ObjType;

pub struct Memory {
    objects: Vec<Rc<ObjType>>
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            objects: Vec::new()
        }
    }
    pub fn push(&mut self, obj: Rc<ObjType>) {
        self.objects.push(obj);
    }
}