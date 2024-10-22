use crate::ir::NamespaceGuard;

use super::{Module, ParserOutput};

impl Module {
    pub fn lower(&self, map: &mut NamespaceGuard, output: &mut ParserOutput) {
        let mut fns = Vec::new();
        for f in &self.functions {
            if let Some(id) = f.lower_header(map, output) {
                fns.push(Some(id));
            } else {
                fns.push(None)
            }
        }
        for (f, id) in self.functions.iter().zip(fns) {
            if let (Some(res), Some(id)) = (f.lower_body(map, output), id) {
                map.write_fn(id, res);
            }
        }
    }
}
