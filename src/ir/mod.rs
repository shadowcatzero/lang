use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::parser::{
    Body, Expr, FileSpan, Function, Ident, Literal, Module, ParserError, ParserErrors, Statement,
};

#[derive(Debug)]
pub enum IRInstruction {
    Li(Var, Literal),
    Mv(Var, Var),
    Not(Var, Var),
    Noti(Var, Literal),
    La(Var, IRIdent),
    Call(FnIdent, Vec<Var>),
}

#[derive(Debug)]
pub struct IRFunction {
    instructions: Vec<IRInstruction>,
}

impl Module {
    pub fn lower(&self, map: &mut Namespace, errors: &mut ParserErrors) {
        for f in &self.functions {
            if let Some(f) = f.as_ref() {
                f.lower(map, errors);
            }
        }
    }
}

impl Function {
    pub fn lower(&self, map: &mut Namespace, errors: &mut ParserErrors) {
        let Some(name) = self.name.as_ref() else {
            return;
        };
        if map.get(name).is_some() {
            errors.add(ParserError {
                msg: format!("Already something named '{:?}'", self.name),
                spans: vec![self.name.span],
            });
        } else {
            let f = map.reserve_fn(self.name.span);
            let mut instructions = Vec::new();
            if let Some(b) = self.body.as_ref() {
                b.lower(map, &mut instructions, errors)
            }
            map.write_fn(name, f, IRFunction { instructions });
        }
    }
}

impl Body {
    pub fn lower(
        &self,
        map: &mut Namespace,
        instructions: &mut Vec<IRInstruction>,
        errors: &mut ParserErrors,
    ) {
        let mut map = map.push();
        for statement in &self.statements {
            let Some(statement) = statement.as_ref() else {
                continue;
            };
            match statement {
                Statement::Let(name_node, expr) => {
                    let Some(name) = name_node.as_ref() else {
                        continue;
                    };
                    let Some(expr) = expr.as_ref() else {
                        continue;
                    };
                    let res = expr.lower(&mut map, instructions, errors);
                    let name = map.add_var(name, name_node.span);
                    if let Some(res) = res {
                        instructions.push(match res {
                            ExprResult::Lit(l) => IRInstruction::Li(name, l),
                            ExprResult::Var(i) => IRInstruction::Mv(name, i),
                            ExprResult::Fn(f) => todo!(),
                        });
                    }
                }
                Statement::Return(e) => todo!(),
                Statement::Expr(expr) => {
                    expr.as_ref().map(|e| e.lower(&mut map, instructions, errors));
                }
            }
        }
    }
}

impl Expr {
    pub fn lower(
        &self,
        map: &mut Namespace,
        instructions: &mut Vec<IRInstruction>,
        errors: &mut ParserErrors,
    ) -> Option<ExprResult> {
        match self {
            Expr::Lit(l) => Some(ExprResult::Lit(l.as_ref()?.clone())),
            Expr::Ident(i) => {
                let Some(id) = map.get(i.as_ref()?) else {
                    errors.add(ParserError::identifier_not_found(i));
                    return None;
                };
                match id.ty() {
                    IdentTypeMatch::Var(var) => Some(ExprResult::Var(var)),
                    IdentTypeMatch::Fn(f) => Some(ExprResult::Fn(f)),
                }
            }
            Expr::BinaryOp(_, _, _) => todo!(),
            Expr::UnaryOp(op, e) => {
                let res = e.as_ref()?.lower(&mut map, instructions, errors)?;
                let res = match op {
                    crate::parser::UnaryOperator::Not => {
                        let temp = map.reserve_var(e.span);
                        match res {
                            ExprResult::Lit(l) => instructions.push(IRInstruction::Noti(temp, l)),
                            ExprResult::Var(i) => instructions.push(IRInstruction::Not(temp, i)),
                            ExprResult::Fn(_) => {
                                errors.add(ParserError::from_span(
                                    e.span,
                                    "Cannot call not on a function".to_string(),
                                ));
                                return None;
                            }
                        }
                        temp
                    }
                    crate::parser::UnaryOperator::Ref => todo!(),
                };
                Some(ExprResult::Var(res))
            }
            Expr::Block(_) => todo!(),
            Expr::Call(e, args) => {
                let e = e.as_ref()?.lower(&mut map, instructions, errors);
                let args = args.iter().map(|a| a.as_ref()?.lower(map, instructions, errors)).collect();
                if let Some(r) = e {
                    let fun = match r {
                        ExprResult::Lit(literal) => todo!(),
                        ExprResult::Var(var) => todo!(),
                        ExprResult::Fn(f) => {
                            instructions.push(IRInstruction::Call(f, args));
                        },
                    };
                } else {
                    todo!();
                }
            },
            Expr::Group(e) => e.as_ref()?.lower(&mut map, instructions, errors),
        }
    }
}

pub enum ExprResult {
    Lit(Literal),
    Var(Var),
    Fn(FnIdent),
}

#[derive(Debug)]
pub struct Namespace {
    pub fns: Vec<Option<IRFunction>>,
    pub vars: usize,
    pub stack: Vec<HashMap<String, IRIdent>>,
}

impl Namespace {
    pub fn new() -> Self {
        Self {
            fns: Vec::new(),
            vars: 0,
            stack: vec![HashMap::new()],
        }
    }
    pub fn push(&mut self) -> NamespaceGuard {
        self.stack.push(HashMap::new());
        NamespaceGuard(self)
    }
    pub fn get(&self, name: &Ident) -> Option<IRIdent> {
        for map in self.stack.iter().rev() {
            let res = map.get(name.val());
            if res.is_some() {
                return res.copied();
            }
        }
        None
    }
    pub fn reserve_var(&mut self, origin: FileSpan) -> Var {
        let i = self.vars;
        self.vars += 1;
        Var(IRIdent {
            origin,
            ty: IdentType::Var,
            i,
        })
    }
    pub fn reserve_fn(&mut self, origin: FileSpan) -> FnIdent {
        let i = self.fns.len();
        self.fns.push(None);
        FnIdent(IRIdent {
            origin,
            ty: IdentType::Fn,
            i,
        })
    }
    pub fn write_fn(&mut self, name: &Ident, id: FnIdent, f: IRFunction) -> IRIdent {
        self.insert(name, id.0);
        self.fns[id.0.i] = Some(f);
        id.0
    }
    pub fn add_var(&mut self, name: &Ident, origin: FileSpan) -> Var {
        let id = self.reserve_var(origin);
        self.insert(name, id.0);
        id
    }
    fn insert(&mut self, name: &Ident, id: IRIdent) {
        let last = self.stack.len() - 1;
        self.stack[last].insert(name.val().to_string(), id);
    }
}

pub struct NamespaceGuard<'a>(&'a mut Namespace);

impl Drop for NamespaceGuard<'_> {
    fn drop(&mut self) {
        self.0.stack.pop();
    }
}

impl Deref for NamespaceGuard<'_> {
    type Target = Namespace;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NamespaceGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IRIdent {
    origin: FileSpan,
    ty: IdentType,
    i: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct FnIdent(IRIdent);
#[derive(Debug, Clone, Copy)]
pub struct Var(IRIdent);

#[derive(Debug, Clone, Copy)]
pub enum IdentType {
    Var,
    Fn,
}

#[derive(Debug, Clone, Copy)]
pub enum IdentTypeMatch {
    Var(Var),
    Fn(FnIdent),
}

impl IRIdent {
    pub fn ty(self) -> IdentTypeMatch {
        match self.ty {
            IdentType::Var => IdentTypeMatch::Var(Var(self)),
            IdentType::Fn => IdentTypeMatch::Fn(FnIdent(self)),
        }
    }
}
