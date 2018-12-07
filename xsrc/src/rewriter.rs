use self::ContextLookupError::*;
use super::schema::{APIData, RootSchema};
use super::se_parser::{parse_expr, Expr, Param, ParserError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::From;
use std::fmt;
use std::iter::FromIterator;
use std::rc::Rc;

#[derive(Debug)]
pub struct RootAST {
    url: Option<String>,
    klsname: String,
    apisets: HashMap<String, APIDataAST>,
    context: Rc<RefCell<Context>>,
}

#[derive(Debug)]
pub enum APIDataAST {
    APIAST(APIAST),
    APISetAST(APISetAST),
}

#[derive(Debug)]
pub struct APIAST {
    name: String,
    method: String,
    url: ContextValue,
    params: Vec<Param>,
    context: Rc<RefCell<Context>>,
}

#[derive(Debug)]
pub struct APISetAST {
    name: String,
    url: ContextValue,
    apisets: HashMap<String, APIDataAST>,
    context: Rc<RefCell<Context>>,
}

#[derive(Debug, PartialEq)]
pub enum RewriterError {
    ContextLookupError(ContextLookupError),
    ParserError(ParserError)
}

impl From<ContextLookupError> for RewriterError {
    fn from(e: ContextLookupError) -> Self {
        RewriterError::ContextLookupError(e)
    }
}

impl From<ParserError> for RewriterError {
    fn from(e: ParserError) -> Self {
        RewriterError::ParserError(e)
    }
}

#[derive(Debug, PartialEq)]
pub enum ContextLookupError {
    NoSuchMember {
        member: String,
        context_path: Vec<String>,
    },
    EmptyKey {
        context_path: Vec<String>,
    },
    LookupOnValue {
        member: String,
        value: ContextValue,
    },
}

impl ContextLookupError {
    fn display_context_path(path: &[String]) -> String {
        path.join(".")
    }
}

impl fmt::Display for ContextLookupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NoSuchMember {
                member,
                context_path,
            } => write!(
                f,
                "No member \"{}\" at {}",
                member,
                ContextLookupError::display_context_path(&context_path)
            ),
            EmptyKey { context_path } => {
                // This should indicate a bug
                write!(
                    f,
                    "Empty key at {}",
                    ContextLookupError::display_context_path(&context_path)
                )
            }
            LookupOnValue { member, value } => write!(
                f,
                "Attempting to lookup \"{}\" on context value \"{:?}\"",
                member, value
            ),
        }
    }
}

#[derive(Debug)]
pub struct Context {
    name: String,
    parent: Option<Rc<RefCell<Context>>>,
    children: HashMap<String, Rc<RefCell<Context>>>,
    scope: HashMap<String, ContextValue>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ContextValue {
    Expr(Expr, Vec<Param>),
}

impl Context {
    fn new(name: String, parent: Option<Rc<RefCell<Context>>>) -> Self {
        Context {
            name,
            parent,
            children: HashMap::new(),
            scope: HashMap::new(),
        }
    }

    fn add_value(&mut self, key: &str, val: ContextValue) {
        self.scope.insert(key.to_string(), val);
    }

    fn add_child(&mut self, key: &str, ctx: Rc<RefCell<Context>>) {
        self.children.insert(key.to_string(), ctx);
    }

    fn path(&self) -> Vec<String> {
        match &self.parent {
            Some(ctx) => {
                let mut ret = ctx.borrow().path().clone();
                ret.push(self.name.to_owned());
                ret
            }
            None => vec![self.name.to_owned()],
        }
    }

    fn lookup_local(&self, key: &str) -> Result<ContextValue, ContextLookupError> {
        if let Some(val) = self.scope.get(key) {
            Ok(val.clone())
        } else {
            Err(ContextLookupError::NoSuchMember {
                member: key.to_string(),
                context_path: self.path(),
            })
        }
    }

    fn lookup(&self, key: &[String]) -> Result<ContextValue, ContextLookupError> {
        if key.len() == 0 {
            Err(ContextLookupError::EmptyKey {
                context_path: self.path(),
            })
        } else {
            if key[0] == "!super" {
                match &self.parent {
                    None => Err(ContextLookupError::NoSuchMember {
                        member: "!super".to_string(),
                        context_path: self.path(),
                    }),
                    Some(ctx) => ctx.borrow().lookup(&key[1..]),
                }
            } else if let Ok(val) = self.lookup_local(&key[0]) {
                if key.len() > 1 {
                    Err(ContextLookupError::LookupOnValue {
                        member: (&key[0]).to_string(),
                        value: val,
                    })
                } else {
                    Ok(val)
                }
            } else if let Some(child_ctx) = self.children.get(&key[0]) {
                child_ctx.borrow().lookup(&key[1..])
            } else {
                Err(ContextLookupError::NoSuchMember {
                    member: (&key[0]).to_string(),
                    context_path: self.path(),
                })
            }
        }
    }
}

fn rewrite_apiset(name: &str, apiset: &APIData, root_ctx: Rc<RefCell<Context>>) -> APIDataAST {
    let mut scope = HashMap::new();
    let ctx = Rc::new(RefCell::new(Context {
        name: name.to_owned(),
        parent: Some(root_ctx),
        children: HashMap::new(),
        scope: scope,
    }));
    match apiset {
        APIData::APISet(schema) => {
            let children = HashMap::from_iter(
                schema
                    .apisets
                    .iter()
                    .map(|(k, v)| (k.to_string(), rewrite_apiset(k, v, Rc::clone(&ctx)))),
            );
            let (expr, params) = parse_expr(schema.s);
            APIDataAST::APISetAST(APISetAST {
                name: name.to_string(),
                url: ContextValue::Expr(expr, params)
                apisets: children,
                context: ctx,
            })
        }
        APIData::API(schema) => APIDataAST::APIAST(APIAST {
            name: name.to_string(),
            method: schema.method.to_string(),
            url: schema.url.to_string(),
            params: Vec::new(),
            context: ctx,
        }),
    }
}

pub fn rewrite(source: RootSchema) -> Result<RootAST, RewriterError> {
    let mut scope = HashMap::new();
    scope.insert("url".to_string(), match source.url {
        Some(ref s) => {
            let (expr, params) = parse_expr(s)?;
            ContextValue::Expr(expr, params)
        },
        None => {
            ContextValue::Expr(
                Expr::Var("url".to_string()),
                vec![Param::new("url", Some("string".to_string()))]
            )
        }
    });
    let root_ctx = Rc::new(RefCell::new(Context {
        name: (&source).klsname.to_owned(),
        parent: None,
        children: HashMap::new(),
        scope,
    }));
    Ok(RootAST {
        url: source.url,
        klsname: source.klsname,
        apisets: HashMap::from_iter(
            (*source.apisets)
                .iter()
                .map(|(k, v)| (k.to_string(), rewrite_apiset(k, v, Rc::clone(&root_ctx)))),
        ),
        context: Rc::clone(&root_ctx),
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use super::super::schema::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn prepare_context() -> Rc<RefCell<Context>> {
        let root_ctx = Rc::new(RefCell::new(Context {
            name: "root".to_string(),
            parent: None,
            children: HashMap::new(),
            scope: hashmap![
                "foo".to_string() => ContextValue::Expr(Expr::Lit("hello".to_string()), Vec::new()),
                "bar".to_string() => ContextValue::Expr(Expr::Lit("world".to_string()), Vec::new()),
            ],
        }));
        let child1_ctx = Rc::new(RefCell::new(Context {
            name: "child1".to_string(),
            parent: Some(root_ctx.clone()),
            children: HashMap::new(),
            scope: hashmap![
                "foo_child1".to_string() => ContextValue::Expr(
                    Expr::Lit("hello_child1".to_string()),
                    Vec::new()
                ),
                "bar_child1".to_string() => ContextValue::Expr(
                    Expr::Lit("world_child1".to_string()),
                    Vec::new()
                ),
            ],
        }));
        let child2_ctx = Rc::new(RefCell::new(Context {
            name: "child2".to_string(),
            parent: Some(root_ctx.clone()),
            children: HashMap::new(),
            scope: hashmap![
                "foo_child2".to_string() => ContextValue::Expr(
                    Expr::Lit("hello_child2".to_string()),
                    Vec::new()
                ),
                "bar_child2".to_string() => ContextValue::Expr(
                    Expr::Lit("world_child2".to_string()),
                    Vec::new()
                ),
            ],
        }));
        root_ctx
            .borrow_mut()
            .add_child("child1", Rc::clone(&child1_ctx));
        root_ctx
            .borrow_mut()
            .add_child("child2", Rc::clone(&child2_ctx));
        return root_ctx;
    }

    #[test]
    fn test_lookup_context() {
        let root_ctx = prepare_context();
        let result = root_ctx
            .borrow()
            .lookup(&["child1".to_string(), "foo_child1".to_string()])
            .unwrap();
        assert_eq!(
            result,
            ContextValue::Expr(Expr::Lit("hello_child1".to_string()), Vec::new())
        );
    }

    #[test]
    fn test_lookup_context_no_such_member() {
        let root_ctx = prepare_context();
        let result = root_ctx
            .borrow()
            .lookup(&["child1".to_string(), "missing".to_string()])
            .err()
            .unwrap();
        assert_eq!(
            result,
            ContextLookupError::NoSuchMember {
                member: "missing".to_string(),
                context_path: vec!["root".to_string(), "child1".to_string()]
            }
        );
    }

    #[test]
    fn test_lookup_super() {
        let root_ctx = prepare_context();
        let ref child1_ctx = root_ctx.borrow().children["child1"];
        let result = child1_ctx
            .borrow()
            .lookup(&["!super".to_string(), "foo".to_string()])
            .unwrap();
        assert_eq!(
            result,
            ContextValue::Expr(Expr::Lit("hello".to_string()), Vec::new())
        )
    }

    #[test]
    fn test_lookup_super_and_child() {
        let root_ctx = prepare_context();
        let ref child1_ctx = root_ctx.borrow().children["child1"];
        let result = child1_ctx
            .borrow()
            .lookup(&[
                "!super".to_string(),
                "child2".to_string(),
                "bar_child2".to_string(),
            ])
            .unwrap();
        assert_eq!(
            result,
            ContextValue::Expr(Expr::Lit("world_child2".to_string()), Vec::new())
        )
    }

    #[test]
    fn test_rewrite() {
        let schema = RootSchema{
            url: Some("http://ratina.org/".to_string()),
            klsname: "RatinaClient".to_string(),
            apisets: APIDataMap(hashmap![
                "ahcro".to_string() => APIData::API(APISchema{
                    method: "GET".to_string(),
                    url: "${!super.url}".to_string()
                })
            ])
        };
        let root_ast = rewrite(schema).unwrap();
        println!("{:?}", root_ast);
    }
}
