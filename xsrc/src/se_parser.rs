//! Parser for string expressions used in schema definitions
use std::fmt;
use self::ParserError::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Member {
    Super,
    Member(String)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Lit(String),
    Concat(Box<Expr>, Box<Expr>),
    Ref(Vec<Member>),
    Var(String)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub name: String,
    pub typ: Option<String>,
}

impl Param {
    pub fn new(name: &str, typ: Option<String>) -> Self {
        Param{
            name: name.to_string(),
            typ
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParserError {
    EmptyExpr,
    UnexpectedToken(String, usize),
    UnexpectedEOF,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EmptyExpr => write!(f, "Expr is empty"),
            UnexpectedToken(s, pos) => write!(f, "Unexpected token \"{}\" at pos {}", s, pos),
            UnexpectedEOF => write!(f, "Unexpected EOF"),
        }
    }
}

fn ident_to_member(s: &str) -> Member {
    if s == "!super" {
        Member::Super
    } else {
        Member::Member(s.to_string())
    }
}

fn parse_ref(s: &str, pos: usize) -> Result<(Expr, usize), ParserError> {
    let mut idents = Vec::new();
    let mut siter = s.chars().skip(pos).enumerate();
    if let Some((inner_pos, ch)) = siter.next() {
        if ch != '{' {
            return Err(ParserError::UnexpectedToken(ch.to_string(), pos + inner_pos));
        }
    } else {
        return Err(ParserError::UnexpectedEOF)
    }
    let mut curr_ident = String::new();
    let inner_pos = loop {
        if let Some((inner_pos, ch)) = siter.next() {
            match ch {
                '\\' => return Err(ParserError::UnexpectedToken(ch.to_string(), pos + inner_pos)),
                '}' => {
                    if curr_ident.len() == 0 {
                        return Err(ParserError::UnexpectedToken(ch.to_string(), pos + inner_pos))
                    } else {
                        idents.push(ident_to_member(&curr_ident));
                    }
                    break inner_pos + 1;
                },
                '.' => {
                    if curr_ident.len() == 0 {
                        return Err(ParserError::UnexpectedToken(ch.to_string(), pos + inner_pos))
                    } else {
                        idents.push(ident_to_member(&curr_ident));
                        curr_ident = String::new();
                    }
                },
                _ => {
                    curr_ident.push(ch);
                }
            }
        } else {
            return Err(ParserError::UnexpectedEOF)
        }
    };
    Ok((Expr::Ref(idents), pos + inner_pos))
}

fn parse_param(s: &str, pos: usize) -> Result<(Expr, Param, usize), ParserError> {
    let mut var = String::new();
    let mut typ = String::new();
    let mut in_var = true;
    let mut siter = s.chars().skip(pos).enumerate();
    let inner_pos = loop {
        if let Some((inner_pos, ch)) = siter.next() {
            match ch {
                '>' => {
                    if var.len() == 0 {
                        return Err(ParserError::UnexpectedToken(ch.to_string(), pos + inner_pos))
                    }
                    break inner_pos + 1
                },
                ':' => {
                    if var.len() == 0 {
                        return Err(ParserError::UnexpectedToken(ch.to_string(), pos + inner_pos))
                    }
                    if !in_var {
                        return Err(ParserError::UnexpectedToken(ch.to_string(), pos + inner_pos))
                    }
                    in_var = false;
                },
                _ => {
                    if in_var {
                        var.push(ch);
                    } else {
                        typ.push(ch);
                    }
                }
            }
        } else {
            return Err(ParserError::UnexpectedEOF)
        }
    };
    if !in_var && typ.len() == 0 { // Caught ':' but no succeeding type
        Err(ParserError::UnexpectedEOF)
    } else {
        Ok(
            (
                Expr::Var(var.to_string()),
                Param{ name: var, typ: if typ.len() == 0 { None } else { Some(typ)} },
                pos + inner_pos
            ),
        )
    }
}

fn collect_exprs(mut exprs: Vec<Expr>) -> Result<Expr, ParserError> {
    if exprs.len() == 0 {
        Err(ParserError::EmptyExpr)
    } else {
        // TODO: Fix this ugly code!
        let mut result = exprs.drain(0..1).next().unwrap();
        for expr in exprs.drain(..) {
            result = Expr::Concat(Box::new(result), Box::new(expr));
        }
        Ok(result)
    }
}

pub fn parse_expr(s: &str) -> Result<(Expr, Vec<Param>), ParserError> {
    let mut exprs = Vec::new();
    let mut params = Vec::new();
    let mut siter = s.chars().enumerate().skip(0);
    let mut curr_str = String::new();
    loop {
        if let Some((pos, ch)) = siter.next() {
            match ch {
                '$' => {
                    if curr_str.len() != 0 {
                        exprs.push(Expr::Lit(curr_str));
                        curr_str = String::new();
                    }
                    let (expr, pos) = parse_ref(s, pos+1)?;
                    exprs.push(expr);
                    siter = s.chars().enumerate().skip(pos);
                },
                '<' => {
                    if curr_str.len() != 0 {
                        exprs.push(Expr::Lit(curr_str));
                        curr_str = String::new();
                    }
                    let (expr, param, pos) = parse_param(s, pos + 1)?;
                    exprs.push(expr);
                    params.push(param);
                    siter = s.chars().enumerate().skip(pos);
                },
                '\\' => {
                    if let Some((_, ch)) = siter.next() {
                        curr_str.push(ch);
                    } else {
                        return Err(ParserError::UnexpectedEOF)
                    }
                }
                _ => {
                    curr_str.push(ch);
                }
            }
        } else {
            if curr_str.len() != 0 {
                exprs.push(Expr::Lit(curr_str));
            }
            break;
        }
    }
    let result = collect_exprs(exprs)?;
    Ok((result, params))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ref() {
        let some_ref = "{a.$b.c}";
        let result = parse_ref(&some_ref, 0);
        let member = (&["a", "$b", "c"]).iter().map(|s| Member::Member(s.to_string())).collect::<Vec<_>>();
        assert_eq!(result.unwrap().0, Expr::Ref(member));
    }

    #[test]
    fn test_parse_ref_malformed() {
        let some_ref = "{.hello}";
        let result = parse_ref(&some_ref, 0);
        let err = result.err().unwrap();
        assert_eq!(err, ParserError::UnexpectedToken('.'.to_string(), 1));
    }

    #[test]
    fn test_parse_ref_malformed2() {
        let some_ref = "{hello..world}";
        let result = parse_ref(&some_ref, 0);
        let err = result.err().unwrap();
        assert_eq!(err, ParserError::UnexpectedToken('.'.to_string(), 7));
    }

    #[test]
    fn test_parse_ref_unterminated() {
        let some_ref = "{";
        let result = parse_ref(&some_ref, 0);
        let err = result.err().unwrap();
        assert_eq!(err, ParserError::UnexpectedEOF);
    }

    #[test]
    fn test_parse_param() {
        let some_param = "hello:world>";
        let result = parse_param(&some_param, 0);
        let (expr, param, pos) = result.unwrap();
        assert_eq!(expr, Expr::Var("hello".to_string()));
        assert_eq!(param, Param{ name: "hello".to_string(), typ: Some("world".to_string())});
        assert_eq!(pos, 12);
    }

    #[test]
    fn test_parse_param_no_type() {
        let some_param = "hello>";
        let result = parse_param(&some_param, 0);
        let (expr, param, pos) = result.unwrap();
        assert_eq!(expr, Expr::Var("hello".to_string()));
        assert_eq!(param, Param{ name: "hello".to_string(), typ: None});
        assert_eq!(pos, 6);
    }

    #[test]
    fn test_parse_param_no_var() {
        let some_param = ":world>";
        let result = parse_param(&some_param, 0);
        let err = result.err().unwrap();
        assert_eq!(err, ParserError::UnexpectedToken(':'.to_string(), 0));
    }

    #[test]
    fn test_parse_param_colon_no_type() {
        let some_param = "hello:>";
        let result = parse_param(&some_param, 0);
        let err = result.err().unwrap();
        assert_eq!(err, ParserError::UnexpectedEOF);
    }

    #[test]
    fn test_collect_exprs() {
        let exprs = vec![
            Expr::Lit("Hello".to_string()),
            Expr::Var("World".to_string()),
            Expr::Var("Xiaosi".to_string()),
        ];
        let expected = Expr::Concat(
            Box::new(Expr::Concat(
                Box::new(Expr::Lit("Hello".to_string())),
                Box::new(Expr::Var("World".to_string())),
            )),
            Box::new(Expr::Var("Xiaosi".to_string())),
        );
        assert_eq!(collect_exprs(exprs).unwrap(), expected);
    }

    #[test]
    fn test_parse_expr() {
        let s = "abc${!super.def}<id:gg>";
        let result = parse_expr(s);
        let (expr, params) = result.unwrap();
        assert_eq!(expr, Expr::Concat(
            box Expr::Concat(
                box Expr::Lit("abc".to_string()),
                box Expr::Ref(vec![Member::Super, Member::Member("def".to_string())])
            ),
            box Expr::Var("id".to_string())
        ));
        assert_eq!(params, vec![Param{ name: "id".to_string(), typ: Some("gg".to_string()) }]);
    }

    #[test]
    fn test_parse_expr_no_var() {
        let s = "abc${super.def}<:gg>";
        let result = parse_expr(s);
        let err = result.err().unwrap();
        assert_eq!(err, ParserError::UnexpectedToken(':'.to_string(), 16))
    }
}
