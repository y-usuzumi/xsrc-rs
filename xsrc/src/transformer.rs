use super::schema::{APIData, APIDataMap, RootSchema};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct Ident(String);

#[derive(Debug, PartialEq)]
pub enum Expr {
    Lit(String),
    Var(Ident),
    Concat(Box<Expr>, Box<Expr>),
    Member(Ident, Ident),
}

pub struct RootAST {
    url: Option<String>,
    klsname: Expr,
    apisets: HashMap<String, APIDataAST>,
    context: Rc<Context>,
}

pub enum APIDataAST {
    APIAST(APIAST),
    APISetAST(APISetAST),
}

pub struct Param {
    name: String,
    typ: String,
}

pub struct APIAST {
    name: String,
    method: String,
    url: String,
    params: Vec<Param>,
    context: Rc<Context>,
}

pub struct APISetAST {
    name: String,
    apisets: HashMap<String, APIDataAST>,
    context: Rc<Context>,
}

struct Context {
    parent: Option<Rc<Context>>,
    scope: HashMap<String, String>,
}

impl Context {
    fn build(parent: Option<Rc<Context>>) -> Context {
        panic!("Not implemented")
    }
    fn lookup(&self, key: &str) -> Result<String, String> {
        if let Some(val) = self.scope.get(key) {
            Ok(val.to_string())
        } else {
            match &self.parent {
                Some(s) => s.lookup(key),
                None => Err(format!("{} is not in scope", key)),
            }
        }
    }
    pub fn rewrite(&self, expr: &str) -> String {
        panic!("Not implemented");
    }
}

#[derive(Debug)]
pub struct ExprParseError {}

fn is_ident_char(ch: char) -> bool {
    // FIXME: More accurate definition of identifier char
    match ch {
        '{' | '}' | '.' | '"' | '\'' => true,
        _ => false,
    }
}

fn parse_member(s: &str) -> Result<(Expr, &str), ExprParseError> {
    panic!("Not implemented");
}

fn parse_var(s: &str) -> Result<(Expr, Param, &str), ExprParseError> {
    let mut var = String::new();
    let mut typ = String::new();
    let mut in_var = true;
    let mut pos = 0;
    for (idx, ch) in s.chars().enumerate() {
        pos = idx;
        if in_var {
            if ch == ':' {
                in_var = false;
                continue;
            }
            var.push(ch);
        } else {
            typ.push(ch);
        }
    }
    Ok((
        Expr::Var(Ident(var.to_string())),
        Param {
            name: var,
            typ: typ,
        },
        &s[pos..],
    ))
}

fn parse_string(s: &str) -> Result<(Expr, &str), ExprParseError> {
    panic!("Not implemented");
}

fn collect_exprs(mut exprs: Vec<Expr>) -> Result<Expr, ExprParseError> {
    if exprs.len() == 0 {
        Err(ExprParseError {})
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

fn parse_expr(s: &str) -> Result<(Expr, Vec<Param>), ExprParseError> {
    let mut exprs = Vec::new();
    let mut params = Vec::new();
    let mut remaining_s = s;
    while remaining_s.len() > 0 {
        match s.chars().next().unwrap() {
            '$' => {
                let (expr, remaining_s0) = parse_member(&s[1..])?;
                exprs.push(expr);
                remaining_s = remaining_s0;
            }
            '<' => {
                let (expr, param, remaining_s0) = parse_var(&s[1..])?;
                exprs.push(expr);
                params.push(param);
                remaining_s = remaining_s0;
            }
            _ => {
                let (expr, remaining_s0) = parse_string(s)?;
                exprs.push(expr);
                remaining_s = remaining_s0;
            }
        }
    }
    let result = collect_exprs(exprs)?;
    Ok((result, params))
}

// fn transform_apiset(name: &str, apiset: &APIData) -> APIDataAST {
//     match apiset {
//         APIData::APISet(schema) => {
//             let children = HashMap::from_iter(
//                 schema.apisets.iter().map(|(k, v)| (k.to_string(), parse_apiset(k, v)))
//             );
//             APIDataAST::APISetAST(APISetAST{
//                 name: name.to_string(),
//                 apisets: children
//             })
//         },
//         APIData::API(schema) => {
//             APIDataAST::APIAST(APIAST{
//                 name: name.to_string(),
//                 method: schema.method.to_string(),
//                 url: schema.url.to_string(),
//                 params: Vec::new()
//             })
//         }
//     }
// }

// pub fn transform(source: RootSchema) -> RootAST {
//     RootAST {
//         url: source.url,
//         klsname: source.klsname,
//         apisets: HashMap::from_iter((*source.apisets).iter().map(|(k, v)| (k.to_string(), parse_apiset(k, v))))
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_exprs() {
        let exprs = vec![
            Expr::Lit("Hello".to_string()),
            Expr::Var(Ident("World".to_string())),
            Expr::Var(Ident("Xiaosi".to_string())),
        ];
        let expected = Expr::Concat(
            Box::new(Expr::Concat(
                Box::new(Expr::Lit("Hello".to_string())),
                Box::new(Expr::Var(Ident("World".to_string()))),
            )),
            Box::new(Expr::Var(Ident("Xiaosi".to_string()))),
        );
        assert_eq!(collect_exprs(exprs).unwrap(), expected);
    }
}
