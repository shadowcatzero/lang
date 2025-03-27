use crate::ir::NamespaceGuard;

use super::{PModule, CompilerOutput};

impl PModule {
    pub fn lower(&self, map: &mut NamespaceGuard, output: &mut CompilerOutput) {
        for s in &self.structs {
            s.lower(map, output);
        }
        let mut fns = Vec::new();
        for f in &self.functions {
            if let Some(id) = f.lower_header(map, output) {
                fns.push(Some(id));
            } else {
                fns.push(None)
            }
        }
        for (f, id) in self.functions.iter().zip(fns) {
            if let Some(id) = id {
                if let Some(res) = f.lower_body(id, map, output) {
                    map.write_fn(id, res);
                }
            }
        }
    }
}
