use std::collections::HashMap;

pub struct Program {
    data: Vec<u8>,
    ro_map: HashMap<String, usize>,
}

impl Program {
    pub fn new(data: HashMap<String, Vec<u8>>) -> Self {
        let mut ro_data = Vec::new();
        let mut ro_map = HashMap::new();
        for (key, val) in data {
            ro_map.insert(key, ro_data.len());
            ro_data.extend(val);
        }
        Self {
            data: ro_data,
            ro_map,
        }
    }
}

pub fn create_program<I: Instr>(
    ro_data: HashMap<String, Vec<u8>>,
    functions: Vec<Function<I>>,
) -> Vec<u8> {
    let mut data = Vec::new();
    let mut ro_map = HashMap::new();
    for (key, val) in ro_data {
        ro_map.insert(key, data.len());
        data.extend(val);
    }
    // let mut fn_map = HashMap::new();
    for fun in functions {
        for i in fun.instructions {
            data.extend(i.to_le_bytes());
        }
    }
    data
}

pub struct Function<I: Instr> {
    label: String,
    instructions: Vec<I>,
}

pub trait Instr {
    fn to_le_bytes(&self) -> impl IntoIterator<Item = u8>;
}

struct SymbolInstr {
    i: usize
}
