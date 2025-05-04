use crate::{
    common::{CompilerMsg, CompilerOutput},
    ir::ID,
};

use super::{KindTy, Origin, StructID, TypeID, UProgram};

pub fn report_errs(p: &UProgram, output: &mut CompilerOutput, errs: Vec<ResErr>) {
    for err in errs {
        match err {
            ResErr::Type {
                dst,
                src,
                errs,
                origin,
            } => {
                let mut msg = type_assign_err(p, dst, src);
                for inner in errs {
                    if inner.dst != dst && inner.src != src {
                        msg.push_str("\n    ");
                        msg.push_str(&type_assign_err(p, inner.dst, inner.src));
                    }
                }
                output.err(CompilerMsg::new(msg, origin));
            }
            ResErr::NotCallable { origin, ty } => {
                output.err(CompilerMsg::new(
                    format!("Cannot call type '{}'", p.type_name(ty)),
                    origin,
                ));
            }
            ResErr::CannotDeref { origin, ty } => {
                output.err(CompilerMsg::new(
                    format!("Cannot dereference type '{}'", p.type_name(ty)),
                    origin,
                ));
            }
            ResErr::CondType { origin, ty } => {
                output.err(CompilerMsg::new(
                    format!("Condition types must be '64'; found '{}'", p.type_name(ty)),
                    origin,
                ));
            }
            ResErr::BadControlFlow { origin, op } => {
                output.err(CompilerMsg::new(
                    format!("Cannot {} here (outside of loop)", op.str()),
                    origin,
                ));
            }
            ResErr::MissingField { origin, id, name } => {
                output.err(CompilerMsg::new(
                    format!(
                        "Missing field '{name}' in creation of struct '{}'",
                        p.structs[id].name
                    ),
                    origin,
                ));
            }
            ResErr::UnknownField { origin, id, name } => {
                output.err(CompilerMsg::new(
                    format!("Unknown field '{name}' in struct '{}'", p.structs[id].name),
                    origin,
                ));
            }
            ResErr::NoReturn { fid } => output.err(CompilerMsg::new(
                format!("Function must return a value"),
                p.fns[fid].origin,
            )),
            ResErr::GenericCount {
                origin,
                expected,
                found,
            } => output.err(CompilerMsg::new(
                if expected == 0 {
                    format!("No generic arguments expected")
                } else {
                    format!("Expected {expected} generic arguments, found {found}")
                },
                origin,
            )),
            ResErr::KindMismatch {
                origin,
                found,
                expected,
                id,
            } => output.err(CompilerMsg::new(
                {
                    let name = match found {
                        KindTy::Type => &p.type_name(ID::new(id)),
                        KindTy::Var => &p.vars[id].name,
                        KindTy::Struct => &p.structs[id].name,
                    };
                    format!(
                        "Expected {}, found {} '{}'",
                        expected.str(),
                        found.str(),
                        name
                    )
                },
                origin,
            )),
            ResErr::UnexpectedField { origin } => {
                output.err(CompilerMsg::new(format!("Unexpected fields here"), origin))
            }
        }
    }
}

pub enum ResErr {
    KindMismatch {
        origin: Origin,
        expected: KindTy,
        found: KindTy,
        id: usize,
    },
    UnexpectedField {
        origin: Origin,
    },
    GenericCount {
        origin: Origin,
        expected: usize,
        found: usize,
    },
    NotCallable {
        origin: Origin,
        ty: TypeID,
    },
    CannotDeref {
        origin: Origin,
        ty: TypeID,
    },
    CondType {
        origin: Origin,
        ty: TypeID,
    },
    NoReturn {
        fid: usize,
    },
    BadControlFlow {
        op: ControlFlowOp,
        origin: Origin,
    },
    MissingField {
        origin: Origin,
        id: StructID,
        name: String,
    },
    UnknownField {
        origin: Origin,
        id: StructID,
        name: String,
    },
    Type {
        dst: TypeID,
        src: TypeID,
        errs: Vec<TypeMismatch>,
        origin: Origin,
    },
}

pub enum ControlFlowOp {
    Break,
    Continue,
}

impl ControlFlowOp {
    pub fn str(&self) -> &'static str {
        match self {
            ControlFlowOp::Break => "break",
            ControlFlowOp::Continue => "continue",
        }
    }
}

pub struct TypeMismatch {
    pub dst: TypeID,
    pub src: TypeID,
}

pub fn type_assign_err(p: &UProgram, dst: TypeID, src: TypeID) -> String {
    format!(
        "Cannot assign type {} to {}",
        p.type_name(src),
        p.type_name(dst)
    )
}
