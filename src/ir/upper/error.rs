use crate::common::{CompilerMsg, CompilerOutput, FileSpan};

use super::{IRUProgram, Type};

impl CompilerOutput {
    pub fn check_assign(&mut self, p: &IRUProgram, src: &Type, dest: &Type, span: FileSpan) {
        // TODO: spans
        if src != dest {
            self.err(CompilerMsg {
                msg: format!(
                    "Cannot assign type '{}' to '{}'",
                    p.type_name(src),
                    p.type_name(dest)
                ),
                spans: vec![span],
            });
        }
    }
}
