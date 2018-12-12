use self::ContextLookupError::*;
use self::TransformerError::*;
use super::schema::{APIData, RootSchema};
pub use super::se_parser::Param;
use super::se_parser::{parse_expr, Expr, Member, ParserError};
use linked_hash_map::LinkedHashMap;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::From;
use std::fmt;
use std::iter::FromIterator;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
}

impl HttpMethod {
    fn from_str(s: &str) -> Self {
        match s {
            "get" | "GET" => HttpMethod::GET,
            "post" | "POST" => HttpMethod::POST,
            "put" | "PUT" => HttpMethod::PUT,
            "delete" | "DELETE" => HttpMethod::DELETE,
            "head" | "HEAD" => HttpMethod::HEAD,
            "options" | "OPTIONS" => HttpMethod::OPTIONS,
            "patch" | "PATCH" => HttpMethod::PATCH,
            _ => panic!("Caught unsupported HTTP method: {}", s),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ContextBoundedRoot {
    pub klsname: String,
    pub url: ContextValue,
    pub bounded_vars: LinkedHashMap<String, Param>,
    pub apisets: LinkedHashMap<String, ContextBoundedAPIData>,
    pub context: Rc<RefCell<Context>>,
}

#[derive(Debug, PartialEq)]
pub enum ContextBoundedAPIData {
    API(ContextBoundedAPI),
    APISet(ContextBoundedAPISet),
}

#[derive(Debug, PartialEq)]
pub struct ContextBoundedAPI {
    pub name: String,
    pub method: HttpMethod,
    pub url: ContextValue,
    pub bounded_vars: LinkedHashMap<String, Param>,
    pub data: LinkedHashMap<String, Param>,
    pub params: LinkedHashMap<String, Param>,
    pub context: Rc<RefCell<Context>>,
}

#[derive(Debug, PartialEq)]
pub struct ContextBoundedAPISet {
    pub name: String,
    pub url: ContextValue,
    pub bounded_vars: LinkedHashMap<String, Param>,
    pub apisets: LinkedHashMap<String, ContextBoundedAPIData>,
    pub context: Rc<RefCell<Context>>,
}

#[derive(Debug, PartialEq)]
pub enum TransformerError {
    ContextLookupError(ContextLookupError),
    ParserError(ParserError),
}

impl From<ContextLookupError> for TransformerError {
    fn from(e: ContextLookupError) -> Self {
        TransformerError::ContextLookupError(e)
    }
}

impl From<ParserError> for TransformerError {
    fn from(e: ParserError) -> Self {
        TransformerError::ParserError(e)
    }
}

impl fmt::Display for TransformerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContextLookupError(e) => write!(f, "Context lookup error: {}", e),
            ParserError(e) => write!(f, "Context lookup error: {}", e),
        }
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

#[derive(Debug, PartialEq)]
pub struct Context {
    name: String,
    parent: Option<Rc<RefCell<Context>>>,
    children: HashMap<String, Rc<RefCell<Context>>>,
    scope: HashMap<String, ContextValue>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ContextValue {
    Expr(Expr),
}

impl Context {
    fn new(name: &str, parent: Option<Rc<RefCell<Context>>>) -> Self {
        Context {
            name: name.to_string(),
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
                ret.push(self.name.to_string());
                ret
            }
            None => vec![self.name.to_string()],
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

fn transform_apiset(
    name: &str,
    apiset: &APIData,
    root_ctx: Rc<RefCell<Context>>,
) -> Result<ContextBoundedAPIData, TransformerError> {
    let mut scope = HashMap::new();
    let ctx = Rc::new(RefCell::new(Context {
        name: name.to_string(),
        parent: Some(root_ctx),
        children: HashMap::new(),
        scope: scope,
    }));
    match apiset {
        APIData::APISet(schema) => {
            let mut children = LinkedHashMap::new();
            for (k, v) in schema.apisets.iter() {
                let child = transform_apiset(k, v, Rc::clone(&ctx))?;
                children.insert(k.to_string(), child);
            }
            let (expr, mut bounded_vars) = parse_expr(&schema.url)?;
            Ok(ContextBoundedAPIData::APISet(ContextBoundedAPISet {
                name: name.to_string(),
                url: ContextValue::Expr(expr),
                bounded_vars,
                apisets: children,
                context: ctx,
            }))
        }
        APIData::API(schema) => {
            let (expr, mut bounded_vars) = parse_expr(&schema.url)?;
            for (name, typ) in &schema.params {
                let p = Param {
                    name: name.to_string(),
                    typ: typ.clone(),
                };
                if bounded_vars.insert(name.to_string(), p).is_some() {
                    return Err(TransformerError::from(ParserError::DuplicateParam(
                        name.to_string(),
                    )));
                }
            }
            for (name, typ) in &schema.data {
                let p = Param {
                    name: name.to_string(),
                    typ: typ.clone(),
                };
                if bounded_vars.insert(name.to_string(), p).is_some() {
                    return Err(TransformerError::from(ParserError::DuplicateParam(
                        name.to_string(),
                    )));
                }
            }
            let data = LinkedHashMap::from_iter(schema.data.iter().map(|(k, v)| {
                (
                    k.to_string(),
                    Param {
                        name: k.to_string(),
                        typ: v.clone(),
                    },
                )
            }));
            let params = LinkedHashMap::from_iter(schema.params.iter().map(|(k, v)| {
                (
                    k.to_string(),
                    Param {
                        name: k.to_string(),
                        typ: v.clone(),
                    },
                )
            }));
            Ok(ContextBoundedAPIData::API(ContextBoundedAPI {
                name: name.to_string(),
                method: HttpMethod::from_str(&schema.method),
                url: ContextValue::Expr(expr),
                bounded_vars,
                data,
                params,
                context: ctx,
            }))
        }
    }
}

pub fn transform(source: RootSchema) -> Result<ContextBoundedRoot, TransformerError> {
    let scope = HashMap::new();
    let url: ContextValue;
    let mut bounded_vars = LinkedHashMap::new();
    match source.url {
        Some(ref s) => {
            let (expr, vars) = parse_expr(s)?;
            url = ContextValue::Expr(expr);
            bounded_vars.extend(vars);
        }
        None => {
            let url_param = Param::new("url", Some("string".to_string()));
            bounded_vars.insert("url".to_string(), url_param);
            url = ContextValue::Expr(Expr::Var("url".to_string()));
        }
    }
    let root_ctx = Rc::new(RefCell::new(Context {
        name: (&source).klsname.to_string(),
        parent: None,
        children: HashMap::new(),
        scope,
    }));
    let mut apisets = LinkedHashMap::new();
    for (k, v) in source.apisets.iter() {
        let child = transform_apiset(k, v, Rc::clone(&root_ctx))?;
        apisets.insert(k.to_string(), child);
    }
    Ok(ContextBoundedRoot {
        klsname: source.klsname,
        url,
        bounded_vars,
        apisets,
        context: Rc::clone(&root_ctx),
    })
}

#[cfg(test)]
pub mod tests {
    use super::super::schema::*;
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn prepare_context() -> Rc<RefCell<Context>> {
        let root_ctx = Rc::new(RefCell::new(Context {
            name: "root".to_string(),
            parent: None,
            children: HashMap::new(),
            scope: hashmap![
                "foo".to_string() => ContextValue::Expr(Expr::Lit("hello".to_string())),
                "bar".to_string() => ContextValue::Expr(Expr::Lit("world".to_string())),
            ],
        }));
        let child1_ctx = Rc::new(RefCell::new(Context {
            name: "child1".to_string(),
            parent: Some(root_ctx.clone()),
            children: HashMap::new(),
            scope: hashmap![
                "foo_child1".to_string() => ContextValue::Expr(Expr::Lit("hello_child1".to_string())),
                "bar_child1".to_string() => ContextValue::Expr(Expr::Lit("world_child1".to_string()))
            ],
        }));
        let child2_ctx = Rc::new(RefCell::new(Context {
            name: "child2".to_string(),
            parent: Some(root_ctx.clone()),
            children: HashMap::new(),
            scope: hashmap![
                "foo_child2".to_string() => ContextValue::Expr(
                    Expr::Lit("hello_child2".to_string()),
                ),
                "bar_child2".to_string() => ContextValue::Expr(
                    Expr::Lit("world_child2".to_string()),
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
            ContextValue::Expr(Expr::Lit("hello_child1".to_string()))
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
        assert_eq!(result, ContextValue::Expr(Expr::Lit("hello".to_string())))
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
            ContextValue::Expr(Expr::Lit("world_child2".to_string()))
        )
    }

    #[test]
    fn test_transform() {
        let schema = RootSchema {
            url: Some("http://ratina.org/<id:int>".to_string()),
            klsname: "RatinaClient".to_string(),
            apisets: APIDataMap(linked_hashmap![
                "ahcro".to_string() => APIData::API(APISchema{
                    method: "GET".to_string(),
                    url: "${!super.url}/<ahcroId:uuid>".to_string(),
                    params: LinkedHashMap::new(),
                    data: LinkedHashMap::new()
                }),
                "ratincren".to_string() => APIData::APISet(APISetSchema{
                    url: "${!super.url}/ratincren".to_string(),
                    apisets: APIDataMap(linked_hashmap![
                        "get".to_string() => APIData::API(APISchema{
                            method: "GET".to_string(),
                            url: "${!super.url}/<name:string>".to_string(),
                            params: LinkedHashMap::new(),
                            data: LinkedHashMap::new()
                        })
                    ])
                })
            ]),
        };
        let root_ast = transform(schema).unwrap();
        let root_ctx = Rc::new(RefCell::new(Context::new("RatinaClient", None)));
        let ahcro_ctx = Rc::new(RefCell::new(Context::new("ahcro", Some(root_ctx.clone()))));
        let ratincren_ctx = Rc::new(RefCell::new(Context::new(
            "ratincren",
            Some(root_ctx.clone()),
        )));
        let ratincren_get_ctx = Rc::new(RefCell::new(Context::new(
            "get",
            Some(ratincren_ctx.clone()),
        )));
        assert_eq!(
            root_ast,
            ContextBoundedRoot {
                klsname: "RatinaClient".to_string(),
                url: ContextValue::Expr(Expr::Concat(
                    box Expr::Lit("http://ratina.org/".to_string()),
                    box Expr::Var("id".to_string())
                )),
                bounded_vars: linked_hashmap![
                    "id".to_string() => Param {
                        name: "id".to_string(),
                        typ: Some("int".to_string())
                    }
                ],
                apisets: linked_hashmap![
                    "ahcro".to_string() => ContextBoundedAPIData::API(ContextBoundedAPI{
                        name: "ahcro".to_string(),
                        method: HttpMethod::from_str("GET"),
                        url: ContextValue::Expr(
                            Expr::Concat(
                                box Expr::Concat(
                                    box Expr::Ref(vec![Member::Super, Member::Member("url".to_string())]),
                                    box Expr::Lit("/".to_string())
                                ),
                                box Expr::Var("ahcroId".to_string())
                            )
                        ),
                        bounded_vars: linked_hashmap![
                            "ahcroId".to_string() => Param{ name: "ahcroId".to_string(), typ: Some("uuid".to_string())}],
                        params: LinkedHashMap::new(),
                        data: LinkedHashMap::new(),
                        context: ahcro_ctx
                    }),
                    "ratincren".to_string() => ContextBoundedAPIData::APISet(ContextBoundedAPISet{
                        name: "ratincren".to_string(),
                        url: ContextValue::Expr(
                            Expr::Concat(
                                box Expr::Ref(vec![Member::Super, Member::Member("url".to_string())]),
                                box Expr::Lit("/ratincren".to_string())
                            )
                        ),
                        bounded_vars: LinkedHashMap::new(),
                        apisets: linked_hashmap![
                            "get".to_string() => ContextBoundedAPIData::API(ContextBoundedAPI{
                                name: "get".to_string(),
                                method: HttpMethod::from_str("GET"),
                                url: ContextValue::Expr(
                                    Expr::Concat(
                                        box Expr::Concat(
                                            box Expr::Ref(vec![Member::Super, Member::Member("url".to_string())]),
                                            box Expr::Lit("/".to_string())
                                        ),
                                        box Expr::Var("name".to_string())
                                    )
                                ),
                                bounded_vars: linked_hashmap![
                                    "name".to_string() => Param{
                                        name: "name".to_string(),
                                        typ: Some("string".to_string())
                                    }
                                ],
                                params: LinkedHashMap::new(),
                                data: LinkedHashMap::new(),
                                context: ratincren_get_ctx
                            })
                        ],
                        context: ratincren_ctx
                    })
                ],
                context: root_ctx
            }
        );
    }
}
