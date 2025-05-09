use crate::common::{CompilerMsg, CompilerOutput};

use super::{
    IdentStatus, KindTy, MemberTy, Origin, Res, ResBase, StructID, Type, TypeID, UProgram,
};

pub fn report_errs(p: &UProgram, output: &mut CompilerOutput, mut errs: Vec<ResErr>) {
    for ident in &p.idents {
        match &ident.status {
            IdentStatus::Unres { path, base } => {
                let mem = path.last().unwrap();
                errs.push(ResErr::UnknownMember {
                    ty: mem.ty,
                    name: mem.name.clone(),
                    origin: mem.origin,
                    parent: base.clone(),
                })
            }
            IdentStatus::Failed(err) => {
                if let Some(err) = err {
                    errs.push(err.clone())
                }
            }
            _ => (),
        }
    }
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
            ResErr::UnknownStructField { origin, id, name } => {
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
            } => output.err(CompilerMsg::new(
                format!("Expected {expected}, found {}", found.display_str(p)),
                origin,
            )),
            ResErr::UnknownMember {
                origin,
                ty,
                name,
                parent,
            } => output.err(CompilerMsg::new(
                format!("Unknown {ty} {name} of {}", parent.display_str(p)),
                origin,
            )),
        }
    }
    for var in &p.vars {
        if let Some(ty) = var.ty() {
            match &p.types[ty] {
                Type::Infer => output.err(CompilerMsg::new(
                    format!("Type of {:?} cannot be inferred", var.name),
                    var.origin,
                )),
                _ => (),
            }
        }
    }
}

#[derive(Clone)]
pub enum ResErr {
    UnknownMember {
        origin: Origin,
        ty: MemberTy,
        name: String,
        parent: ResBase,
    },
    KindMismatch {
        origin: Origin,
        expected: KindTy,
        found: Res,
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
    UnknownStructField {
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
