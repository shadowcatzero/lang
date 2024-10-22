use super::{Node, Parsable, ParserMsg, ParserOutput, Symbol, TokenCursor};

pub fn parse_list_sep<T: Parsable>(
    cursor: &mut TokenCursor,
    errors: &mut ParserOutput,
    sep: Symbol,
    end: Symbol,
) -> Result<Vec<Node<T>>, ParserMsg> {
    let mut vals = Vec::new();
    loop {
        let next = cursor.expect_peek()?;
        if next.is_symbol(end) {
            break;
        }
        let res = Node::parse(cursor, errors);
        vals.push(res.node);
        if res.recover {
            cursor.seek_syms(&[end, sep]);
        }
        let next = cursor.expect_peek()?;
        if !next.is_symbol(sep) {
            break;
        }
        cursor.next();
    }
    cursor.expect_sym(end)?;
    Ok(vals)
}

pub fn parse_list<T: Parsable>(
    cursor: &mut TokenCursor,
    errors: &mut ParserOutput,
    end: Symbol,
) -> Result<Vec<Node<T>>, ParserMsg> {
    parse_list_sep(cursor, errors, Symbol::Comma, end)
}

pub fn parse_list_nosep<T: Parsable>(
    cursor: &mut TokenCursor,
    errors: &mut ParserOutput,
    end: Symbol,
) -> Result<Vec<Node<T>>, ParserMsg> {
    let mut vals = Vec::new();
    loop {
        let next = cursor.expect_peek()?;
        if next.is_symbol(end) {
            break;
        }
        let res = Node::parse(cursor, errors);
        vals.push(res.node);
        if res.recover {
            cursor.seek_sym(end);
        }
    }
    cursor.expect_sym(end)?;
    Ok(vals)
}
