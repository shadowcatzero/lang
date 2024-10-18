use std::collections::HashMap;

use crate::parser::{Body, Expr, Function, Ident, Literal, ParserError, Statement, Unresolved};

#[derive(Debug)]
pub enum IRInstruction {
    Li(IRIdent, Literal),
    Mv(IRIdent, IRIdent),
}

#[derive(Debug)]
pub struct IRFunction {
    instructions: Vec<IRInstruction>,
}

impl Function<Unresolved> {
    pub fn lower(&self, functions: &mut Vec<IRFunction>, map: &mut Namespace) -> Result<(), ParserError> {
        let Ok(name) = &self.name.inner else {
            return Ok(());
        };
        if map.get(name).is_some() {
            Err(ParserError {
                msg: format!("Already something named '{:?}'", self.name),
                spans: vec![self.name.span],
            })
        } else {
            map.add(name);
            let mut instructions = Vec::new();
            self.body.as_ref().map(|b| b.lower(map, &mut instructions));
            functions.push(IRFunction { instructions });
            Ok(())
        }
    }
}

impl Body<Unresolved> {
    pub fn lower(&self, map: &Namespace, instructions: &mut Vec<IRInstruction>) {
        let mut map = map.clone();
        for statement in &self.statements {
            let Ok(statement) = &statement.inner else {
                continue;
            };
            match statement {
                Statement::Let(name, expr) => {
                    let Ok(name) = &name.inner else {
                        continue;
                    };
                    let name = map.add(name);
                    let Ok(expr) = &expr.inner else {
                        continue;
                    };
                    if let Ok(Some(res)) = expr.lower(&map, instructions) {
                        instructions.push(match res {
                            ExprResult::Lit(l) => IRInstruction::Li(name, l),
                            ExprResult::Ident(i) => IRInstruction::Mv(name, i),
                        });
                    }
                }
                Statement::Return(e) => todo!(),
                Statement::Expr(expr) => todo!(),
            }
        }
    }
}

impl Expr<Unresolved> {
    pub fn lower(
        &self,
        map: &Namespace,
        instructions: &mut Vec<IRInstruction>,
    ) -> Result<Option<ExprResult>, String> {
        Ok(match self {
            Expr::Lit(l) => {
                let Ok(l) = &l.inner else {return Ok(None)};
                Some(ExprResult::Lit(l.clone()))
            },
            Expr::Ident(i) => {
                let Ok(i) = &i.inner else {return Ok(None)};
                let Some(id) = map.get(i) else {
                    return Err(format!("Identifier '{:?}' not found", i));
                };
                Some(ExprResult::Ident(id))
            }
            Expr::BinaryOp(_, _, _) => todo!(),
            Expr::UnaryOp(_, _) => todo!(),
            Expr::Block(_) => todo!(),
            Expr::Call(_, _) => todo!(),
            Expr::Group(_) => todo!(),
        })
    }
}

enum ExprResult {
    Lit(Literal),
    Ident(IRIdent),
}

#[derive(Debug, Clone)]
pub struct Namespace {
    pub cur: usize,
    pub map: HashMap<String, IRIdent>,
}

impl Namespace {
    pub fn new() -> Self {
        Self {
            cur: 0,
            map: HashMap::new(),
        }
    }
    pub fn get(&self, name: &Ident) -> Option<IRIdent> {
        self.map.get(name.val()).copied()
    }
    pub fn reserve(&mut self) -> IRIdent {
        let id = IRIdent ( self.cur );
        self.cur += 1;
        id
    }
    pub fn add(&mut self, name: &Ident) -> IRIdent {
        let id = self.reserve();
        self.map.insert(name.val().to_string(), id);
        id
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IRIdent (usize);

