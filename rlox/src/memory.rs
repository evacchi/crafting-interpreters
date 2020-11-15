use std::rc::Rc;
use std::collections::HashMap;
use std::collections::HashSet;

use object::ObjType;
use value::Value;

pub struct Memory {
    objects: Vec<ObjType>,
    globals: HashMap<String, Value>,
    strings: HashSet<Rc<String>>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            objects: Vec::new(),
            globals: HashMap::new(),
            strings: HashSet::new()
        }
    }
    pub fn set_global(&mut self, s: String, value: Value) -> bool {
        let r = self.globals.insert(s, value);
        r.is_none() // return true if key is new
    }

    pub fn get_global(&mut self, s: String) -> Option<&Value> {
        self.globals.get(&s)
    }

    pub fn delete_global(&mut self, s: String) {
        self.globals.remove(&s);
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