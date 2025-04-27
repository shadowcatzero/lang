use crate::common::{CompilerMsg, CompilerOutput, FileSpan};

use super::{UProgram, Type};

impl CompilerOutput {
    pub fn check_assign(&mut self, p: &UProgram, src: &Type, dest: &Type, span: FileSpan) {
        // TODO: spans
        if src != dest {
            if !src.is_resolved() || !dest.is_resolved() {
                return;
            }
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
