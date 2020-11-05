#[derive(Debug,Copy,Clone)]
pub struct Value(pub f64);

impl Value {
    pub fn print(&self) {
        print!("{}", self.0);
    }
}