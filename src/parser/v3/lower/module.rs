use crate::ir::IRUProgram;

use super::{PModule, CompilerOutput};

impl PModule {
    pub fn lower(&self, p: &mut IRUProgram, output: &mut CompilerOutput) {
        for s in &self.structs {
            s.lower(p, output);
        }
        let mut fns = Vec::new();
        for f in &self.functions {
            if let Some(id) = f.lower_header(p, output) {
                fns.push(Some(id));
            } else {
                fns.push(None)
            }
        }
        for (f, id) in self.functions.iter().zip(fns) {
            if let Some(id) = id {
                if let Some(res) = f.lower_body(id, p, output) {
                    p.write_fn(id, res);
                }
            }
        }
    }
}
