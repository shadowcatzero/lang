use std::collections::HashMap;

pub fn create_program<I: Instr>(
    ro_data: HashMap<String, Vec<u8>>,
    functions: Vec<Function<I>>,
) -> (Vec<u8>, Option<u64>) {
    let mut data = Vec::new();
    let mut sym_map = HashMap::new();
    for (key, val) in ro_data {
        sym_map.insert(key, data.len() as u64);
        data.extend(val);
    }
    let mut start = None;
    for fun in functions {
        if fun.label == "_start" {
            start = Some(data.len() as u64);
        }
        sym_map.insert(fun.label, data.len() as u64);
        for i in fun.instructions {
            let pos = data.len() as u64;
            i.push(&mut data, &sym_map, pos);
        }
    }
    (data, start)
}

pub struct Function<I: Instr> {
    pub label: String,
    pub instructions: Vec<I>,
}

pub trait Instr {
    fn push(&self, data: &mut Vec<u8>, ro_map: &HashMap<String, u64>, pos: u64) -> Option<String>;
}

