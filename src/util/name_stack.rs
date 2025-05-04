use std::collections::HashMap;

pub struct NameStack<T>(Vec<HashMap<String, T>>);

impl<T> NameStack<T> {
    pub fn new() -> Self {
        Self(vec![HashMap::new()])
    }
    pub fn search(&self, name: &str) -> Option<&T> {
        for level in self.0.iter().rev() {
            if let Some(v) = level.get(name) {
                return Some(v);
            }
        }
        None
    }
    pub fn push(&mut self) {
        self.0.push(HashMap::new());
    }
    pub fn pop(&mut self) {
        self.0.pop();
    }
}
