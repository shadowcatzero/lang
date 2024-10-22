#[derive(Debug)]
pub enum IRInstruction {
    Li(Var, Literal),
    Mv(Var, Var),
    Not(Var, Var),
    Add(Var, Var, Var),
    La(Var, IRIdent),
    Call(Var, FnIdent, Vec<ExprResult>),
}

#[derive(Debug)]
pub struct IRFunction {
    args: Vec<Var>,
    instructions: Vec<IRInstruction>,
}

impl Module {
    pub fn lower(&self, map: &mut Namespace, errors: &mut ParserErrors) {
        let mut fns = Vec::new();
        for f in &self.functions {
            if let Some(f) = f.as_ref() {
                if let Some(id) = f.reserve(map, errors) {
                    fns.push((id, f));
                }
            }
        }
        for (id, f) in fns {
            f.lower(id, map, errors);
        }
    }
}

impl Function {
    pub fn reserve(&self, map: &mut Namespace, errors: &mut ParserErrors) -> Option<FnIdent> {
        let name = self.name.as_ref()?;
        if let Some(other) = map.get(name) {
            errors.add(ParserError {
                msg: format!("Already {:?} named '{:?}'", other.ty, self.name),
                spans: vec![self.name.span, other.origin],
            });
            None
        } else {
            Some(map.reserve_fn(name, self.name.span))
        }
    }
    pub fn lower(
        &self,
        ident: FnIdent,
        map: &mut Namespace,
        errors: &mut ParserErrors,
    ) -> Option<()> {
        let mut instructions = Vec::new();
        let mut map = map.push();
        let mut args = Vec::new();
        for arg in &self.args {
            args.push(map.def_var(arg.as_ref()?, arg.span)?);
        }
        if let Some(b) = self.body.as_ref() {
            b.lower(&mut map, &mut instructions, errors)
        }
        map.write_fn(ident, IRFunction { instructions, args });
        Some(())
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
                    let res = expr.lower(&mut map, instructions, errors);
                    if let Some(res) = res {
                        match res {
                            ExprResult::Var(v) => map.name_var(name, v),
                            ExprResult::Fn(f) => todo!(),
                        };
                    }
                }
                Statement::Return(e) => todo!(),
                Statement::Expr(expr) => {
                    expr.lower(&mut map, instructions, errors);
                }
            }
        }
    }
}

impl Node<Box<Expr>> {
    pub fn lower(
        &self,
        map: &mut Namespace,
        instructions: &mut Vec<IRInstruction>,
        errors: &mut ParserErrors,
    ) -> Option<ExprResult> {
        self.as_ref()?.lower(self.span, map, instructions, errors)
    }
}

impl Node<Expr> {
    pub fn lower(
        &self,
        map: &mut Namespace,
        instructions: &mut Vec<IRInstruction>,
        errors: &mut ParserErrors,
    ) -> Option<ExprResult> {
        self.as_ref()?.lowerr(self.span, map, instructions, errors)
    }
}

impl Expr {
    pub fn lowerr(
        &self,
        span: FileSpan,
        map: &mut Namespace,
        instructions: &mut Vec<IRInstruction>,
        errors: &mut ParserErrors,
    ) -> Option<ExprResult> {
        match self {
            Expr::Lit(l) => {
                let temp = map.temp_var(span);
                instructions.push(IRInstruction::Li(temp, l.as_ref()?.clone()));
                Some(ExprResult::Var(temp))
            },
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
            Expr::BinaryOp(op, e1, e2) => {
                let res1 = e1.lower(map, instructions, errors)?;
                let res2 = e2.lower(map, instructions, errors)?;
                let (ExprResult::Var(v1), ExprResult::Var(v2)) = (res1, res2) else {
                    errors.add(ParserError::from_span(span, "Cannot operate on functions".to_string()));
                    return None;
                };
                let temp = map.temp_var(span);
                match op {
                    crate::parser::BinaryOperator::Add => {
                        instructions.push(IRInstruction::Add(temp, v1, v2))
                    }
                    crate::parser::BinaryOperator::Sub => todo!(),
                    crate::parser::BinaryOperator::Mul => todo!(),
                    crate::parser::BinaryOperator::Div => todo!(),
                    crate::parser::BinaryOperator::LessThan => todo!(),
                    crate::parser::BinaryOperator::GreaterThan => todo!(),
                    crate::parser::BinaryOperator::Access => todo!(),
                    crate::parser::BinaryOperator::Assign => todo!(),
                }
                Some(ExprResult::Var(temp))
            }
            Expr::UnaryOp(op, e) => {
                let res = e.lower(map, instructions, errors)?;
                let res = match op {
                    crate::parser::UnaryOperator::Not => {
                        let temp = map.temp_var(span);
                        match res {
                            ExprResult::Var(i) => instructions.push(IRInstruction::Not(temp, i)),
                            ExprResult::Fn(_) => {
                                errors.add(ParserError::from_span(
                                    span,
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
                let fe = e.lower(map, instructions, errors);
                let mut nargs = Vec::new();
                for arg in args.iter() {
                    let arg = arg.lower(map, instructions, errors)?;
                    nargs.push(arg);
                }
                if let Some(r) = fe {
                    match r {
                        ExprResult::Fn(f) => {
                            let temp = map.temp_var(span);
                            instructions.push(IRInstruction::Call(temp, f, nargs));
                            Some(ExprResult::Var(temp))
                        }
                        o => {
                            errors.add(ParserError::from_span(
                                span,
                                "Expected function".to_string(),
                            ));
                            None
                        }
                    }
                } else {
                    None
                }
            }
            Expr::Group(e) => e.lower(map, instructions, errors),
        }
    }
}

#[derive(Debug)]
pub enum ExprResult {
    Var(Var),
    Fn(FnIdent),
}

