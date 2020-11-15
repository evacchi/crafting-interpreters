use std::rc::Rc;
use std::collections::HashSet;

use object::ObjType;

pub struct Memory {
    objects: Vec<ObjType>,
    strings: HashSet<Rc<String>>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            objects: Vec::new(),
            strings: HashSet::new()
        }
    }
    pub fn push(&mut self, obj: ObjType) {
        self.objects.push(obj.clone());
        match obj {
            ObjType::String(r) =>  { self.intern(r); }
        }
    }
    pub fn intern(&mut self, obj: Rc<String>) -> Rc<String> {
        match self.strings.get(&obj) {
            None => {
                let item = obj.clone();
                self.strings.insert(item);
                obj
            }
            Some(e) => e.clone(),
        }
    }
}