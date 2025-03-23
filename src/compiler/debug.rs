use crate::ir::Symbol;

pub struct DebugInfo {
    pub sym_labels: Vec<Option<String>>,
    pub ir_lower: Vec<Vec<(usize, String)>>,
}

impl DebugInfo {
    pub fn new(sym_labels: Vec<Option<String>>) -> Self {
        Self {
            ir_lower: Vec::new(),
            sym_labels,
        }
    }

    pub fn push_fn(&mut self, instrs: Vec<(usize, String)>) {
        self.ir_lower.push(instrs);
    }

    pub fn sym_label(&self, s: Symbol) -> Option<&String> {
        self.sym_labels[*s].as_ref()
    }
}
