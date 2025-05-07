use std::collections::HashMap;

pub struct NameStack<T> {
    base: HashMap<String, T>,
    levels: Vec<HashMap<String, T>>,
}

impl<T> NameStack<T> {
    pub fn new() -> Self {
        Self {
            base: HashMap::new(),
            levels: Vec::new(),
        }
    }
    pub fn search(&self, name: &str) -> Option<&T> {
        for level in self.levels.iter().rev() {
            if let Some(v) = level.get(name) {
                return Some(v);
            }
        }
        self.base.get(name)
    }
    pub fn push(&mut self) {
        self.levels.push(HashMap::new());
    }
    pub fn pop(&mut self) {
        self.levels.pop();
    }
    fn cur(&mut self) -> &mut HashMap<String, T> {
        self.levels.last_mut().unwrap_or(&mut self.base)
    }
    pub fn insert(&mut self, name: String, v: T) -> bool {
        let cur = self.cur();
        if cur.contains_key(&name) {
            return true;
        }
        cur.insert(name, v);
        false
    }
    pub fn extend(&mut self, iter: impl Iterator<Item = (String, T)>) {
        for (name, v) in iter {
            self.insert(name, v);
        }
    }
}
