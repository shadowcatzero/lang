use crate::common::{CompilerMsg, CompilerOutput, FileSpan};

use super::{Type, UProgram};

impl CompilerOutput {
    pub fn check_assign(&mut self, p: &UProgram, src: &Type, dest: &Type, span: FileSpan) -> bool {
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
            true
        } else {
            false
        }
    }
}
