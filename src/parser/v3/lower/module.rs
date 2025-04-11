use crate::ir::UProgram;

use super::{CompilerOutput, PModule};

impl PModule {
    pub fn lower(&self, p: &mut UProgram, output: &mut CompilerOutput) {
        let mut structs = Vec::new();
        for s in &self.structs {
            structs.push(s.lower_name(p));
        }
        for (s, id) in self.structs.iter().zip(structs) {
            if let Some(id) = id {
                s.lower(id, p, output);
            }
        }
        let mut fns = Vec::new();
        for f in &self.functions {
            fns.push(f.lower_name(p));
        }
        for (f, id) in self.functions.iter().zip(fns) {
            if let Some(id) = id {
                f.lower(id, p, output)
            }
        }
    }
}
