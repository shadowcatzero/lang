use super::{Node, Parsable, ParserCtx, CompilerMsg, Symbol};

pub fn parse_list_sep<T: Parsable>(
    ctx: &mut ParserCtx,
    sep: Symbol,
    end: Symbol,
) -> Result<Vec<Node<T>>, CompilerMsg> {
    let mut vals = Vec::new();
    loop {
        let next = ctx.expect_peek()?;
        if next.is_symbol(end) {
            break;
        }
        let res = ctx.parse();
        vals.push(res.node);
        if res.recover {
            ctx.seek_syms(&[end, sep]);
        }
        let next = ctx.expect_peek()?;
        if !next.is_symbol(sep) {
            break;
        }
        ctx.next();
    }
    ctx.expect_sym(end)?;
    Ok(vals)
}

pub fn parse_list<T: Parsable>(
    ctx: &mut ParserCtx,
    end: Symbol,
) -> Result<Vec<Node<T>>, CompilerMsg> {
    parse_list_sep(ctx, Symbol::Comma, end)
}

pub fn parse_list_nosep<T: Parsable>(
    ctx: &mut ParserCtx,
    end: Symbol,
) -> Result<Vec<Node<T>>, CompilerMsg> {
    let mut vals = Vec::new();
    loop {
        let next = ctx.expect_peek()?;
        if next.is_symbol(end) {
            break;
        }
        let res = ctx.parse();
        vals.push(res.node);
        if res.recover {
            ctx.seek_sym(end);
        }
    }
    ctx.expect_sym(end)?;
    Ok(vals)
}
