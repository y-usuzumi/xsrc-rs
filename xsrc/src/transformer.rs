use std::fmt;
use super::schema::{APIData, APIDataMap, RootSchema};
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use super::separser::{Expr, Param};
use self::ContextLookupError::*;

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

#[derive(Debug, PartialEq)]
pub enum ContextLookupError {
    NoSuchMember{
        member: String,
        context_path: Vec<String>
    },
    EmptyKey{
        context_path: Vec<String>
    },
    LookupOnValue{
        member: String,
        value: ContextValue
    }
}

impl ContextLookupError {
    fn display_context_path(path: &[String]) -> String {
        path.join(".")
    }
}

impl fmt::Display for ContextLookupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NoSuchMember{member, context_path} => {
                write!(f, "No member \"{}\" at {}", member, ContextLookupError::display_context_path(&context_path))
            },
            EmptyKey{context_path} => {
                // This should indicate a bug
                write!(f, "Empty key at {}", ContextLookupError::display_context_path(&context_path))
            },
            LookupOnValue{member, value} => {
                write!(f, "Attempting to lookup \"{}\" on context value \"{}\"", member, value)
            }
        }
    }
}

pub struct Context {
    name: String,
    parent: Option<Rc<RefCell<Context>>>,
    children: HashMap<String, Rc<RefCell<Context>>>,
    scope: HashMap<String, ContextValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContextValue {
    String(String)
}

impl fmt::Display for ContextValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContextValue::String(s) => write!(f, "{}", s)
        }
    }
}

impl Context {
    fn new(parent: Option<Rc<Context>>) -> Context {
        panic!("Not implemented")
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
            None => vec![self.name.to_owned()]
        }
    }

    fn lookup_local(&self, key: &str) -> Result<ContextValue, ContextLookupError> {
        if let Some(val) = self.scope.get(key) {
            Ok(val.clone())
        } else {
            Err(ContextLookupError::NoSuchMember{ member: key.to_string(), context_path: self.path() })
        }
    }

    fn lookup(&self, key: &[String]) -> Result<ContextValue, ContextLookupError> {
        if key.len() == 0 {
            Err(ContextLookupError::EmptyKey{ context_path: self.path() })
        } else {
            if key[0] == "!super" {
                match &self.parent {
                    None => Err(ContextLookupError::NoSuchMember{ member: "!super".to_string(), context_path: self.path() }),
                    Some(ctx) => ctx.borrow().lookup(&key[1..])
                }
            } else if let Ok(val) = self.lookup_local(&key[0]) {
                if key.len() > 1 {
                    Err(ContextLookupError::LookupOnValue{member: (&key[0]).to_string(), value: val})
                } else {
                    Ok(val)
                }
            } else if let Some(child_ctx) = self.children.get(&key[0]) {
                child_ctx.borrow().lookup(&key[1..])
            } else {
                Err(ContextLookupError::NoSuchMember{ member: (&key[0]).to_string(), context_path: self.path() })
            }
        }
    }

    pub fn rewrite(&self, expr: &str) -> String {
        panic!("Not implemented");
    }
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
pub mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use super::*;

    fn prepare_context() -> Rc<RefCell<Context>> {
        let root_ctx = Rc::new(RefCell::new(Context{
            name: "root".to_string(),
            parent: None,
            children: HashMap::new(),
            scope: hashmap![
                "foo".to_string() => ContextValue::String("hello".to_string()),
                "bar".to_string() => ContextValue::String("world".to_string()),
            ]
        }));
        let child1_ctx = Rc::new(RefCell::new(Context{
            name: "child1".to_string(),
            parent: Some(root_ctx.clone()),
            children: HashMap::new(),
            scope: hashmap![
                "foo_child1".to_string() => ContextValue::String("hello_child1".to_string()),
                "bar_child1".to_string() => ContextValue::String("world_child1".to_string()),
            ]
        }));
        let child2_ctx = Rc::new(RefCell::new(Context{
            name: "child2".to_string(),
            parent: Some(root_ctx.clone()),
            children: HashMap::new(),
            scope: hashmap![
                "foo_child2".to_string() => ContextValue::String("hello_child2".to_string()),
                "bar_child2".to_string() => ContextValue::String("world_child2".to_string()),
            ]
        }));
        root_ctx.borrow_mut().add_child("child1", Rc::clone(&child1_ctx));
        root_ctx.borrow_mut().add_child("child2", Rc::clone(&child2_ctx));
        return root_ctx;
    }

    #[test]
    fn test_lookup_context() {
        let root_ctx = prepare_context();
        let result = root_ctx.borrow().lookup(
            &["child1".to_string(), "foo_child1".to_string()]
        ).unwrap();
        assert_eq!(result, ContextValue::String("hello_child1".to_string()));
    }

    #[test]
    fn test_lookup_context_no_such_member() {
        let root_ctx = prepare_context();
        let result = root_ctx.borrow().lookup(
            &["child1".to_string(), "missing".to_string()]
        ).err().unwrap();
        assert_eq!(result, ContextLookupError::NoSuchMember{
            member: "missing".to_string(),
            context_path: vec!["root".to_string(), "child1".to_string()]
        });
    }

    #[test]
    fn test_lookup_super() {
        let root_ctx = prepare_context();
        let ref child1_ctx = root_ctx.borrow().children["child1"];
        let result = child1_ctx.borrow().lookup(
            &["!super".to_string(), "foo".to_string()]
        ).unwrap();
        assert_eq!(result, ContextValue::String("hello".to_string()))
    }

    #[test]
    fn test_lookup_super_and_child() {
        let root_ctx = prepare_context();
        let ref child1_ctx = root_ctx.borrow().children["child1"];
        let result = child1_ctx.borrow().lookup(
            &["!super".to_string(), "child2".to_string(), "bar_child2".to_string()]
        ).unwrap();
        assert_eq!(result, ContextValue::String("world_child2".to_string()))
    }
}
