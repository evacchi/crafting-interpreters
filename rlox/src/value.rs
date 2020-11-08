#[derive(Debug,Copy,Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64)
}

impl Value {
    pub fn print(&self) {
        print!("{:?}", self);
    }
}