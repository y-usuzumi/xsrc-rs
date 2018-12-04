//! Parser for string expressions used in schema definitions
use std::fmt;
use self::ParserError::*;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Lit(String),
    Concat(Box<Expr>, Box<Expr>),
    Member(Vec<String>),
    Var(String)
}

#[derive(Debug, PartialEq)]
pub struct Param {
    name: String,
    typ: Option<String>,
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
                        idents.push(curr_ident.to_owned());
                    }
                    break inner_pos + 1;
                },
                '.' => {
                    if curr_ident.len() == 0 {
                        return Err(ParserError::UnexpectedToken(ch.to_string(), pos + inner_pos))
                    } else {
                        idents.push(curr_ident.to_owned());
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
    Ok((Expr::Member(idents), pos + inner_pos))
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
                Expr::Var(var.to_owned()),
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
            let new_expr = Expr::Concat(Box::new(result), Box::new(expr));
            result = new_expr;
        }
        Ok(result)
    }
}

// fn parse_expr(s: &str) -> Result<(Expr, Vec<Param>), ParserError> {
//     let mut exprs = Vec::new();
//     let mut params = Vec::new();
//     let pos = 0;
//     let in_s = true;
//     let siter = s.chars().enumerate();
//     loop {
//         if let (pos, ch) = siter.next() {
//             match ch {
//                 '\\' => {
//                     if !in_s {
//                         return Err(ParserError::TokenError('\\', Pos::Pos(pos)));
//                     }
//                     match siter.next() {
//                         None => return Err(ParserError::UnterminatedEscape(Pos::End)),
//                         Some(ch) => panic!("TODO"),
//                     }
//                 }
//                 '$' => {
//                     if in_s {
//                         in_s = false;
//                         let (expr, remaining_s0) = parse_member(siter)?;
//                         exprs.push(expr);
//                         remaining_s = remaining_s0;
//                     } else {
//                         return Err(ParserError::SomeError(""))
//                     }
//                 }
//                 '<' => {
//                     let (expr, param, remaining_s0) = parse_var(&s[1..])?;
//                     exprs.push(expr);
//                     params.push(param);
//                     remaining_s = remaining_s0;
//                 }
//                 _ => {
//                 }
//             }
//         }
//     }
//     let result = collect_exprs(exprs)?;
//     Ok((result, params))
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ref() {
        let some_ref = "{a.$b.c}";
        let result = parse_ref(&some_ref, 0);
        let member = (&["a", "$b", "c"]).iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(result.unwrap().0, Expr::Member(member));
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

    // #[test]
    // fn test_collect_exprs() {
    //     let exprs = vec![
    //         Expr::Lit("Hello".to_string()),
    //         Expr::Var(Ident("World".to_string())),
    //         Expr::Var(Ident("Xiaosi".to_string())),
    //     ];
    //     let expected = Expr::Concat(
    //         Box::new(Expr::Concat(
    //             Box::new(Expr::Lit("Hello".to_string())),
    //             Box::new(Expr::Var(Ident("World".to_string()))),
    //         )),
    //         Box::new(Expr::Var(Ident("Xiaosi".to_string()))),
    //     );
    //     assert_eq!(collect_exprs(exprs).unwrap(), expected);
    // }

    // #[test]
    // fn test_parse_expr() {
    //     let s = "abc${super.def}<id:gg>";
    // }
}
