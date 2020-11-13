use std::rc::Rc;
use std::collections::HashSet;

use object::ObjType;

pub struct Memory {
    objects: Vec<Rc<ObjType>>,
    strings: HashSet<Rc<String>>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            objects: Vec::new(),
            strings: HashSet::new()
        }
    }
    pub fn push(&mut self, obj: Rc<ObjType>) {
        self.objects.push(obj);
    }
    pub fn intern(&mut self, obj: Rc<String>) -> Rc<String> {
        match self.strings.get(&obj) {
            None => {
                self.strings.insert(obj.clone());
                obj
            } 
            Some(&v) => v
        }
    }
}