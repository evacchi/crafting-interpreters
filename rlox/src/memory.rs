use std::collections::HashMap;
use std::collections::HashSet;

use object::ObjType;
use value::Value;

#[derive(Clone)]
pub struct Memory {
    objects: Vec<ObjType>,
    pub globals: HashMap<String, Value>,
    strings: HashSet<String>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            objects: Vec::new(),
            globals: HashMap::new(),
            strings: HashSet::new(),
        }
    }
    pub fn set_global(&mut self, s: String, value: Value) -> bool {
        self.globals.insert(s, value).is_none()
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
            ObjType::String(r) => {
                self.intern(r);
            }
            _ => { /* ignore otherwise */ }
        }
    }
    pub fn intern(&mut self, obj: String) -> String {
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
