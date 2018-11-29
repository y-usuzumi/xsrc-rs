use std::rc::Rc;
use std::collections::HashMap;
use super::schema::RootSchema;

pub struct RootAST {
    url: Option<String>,
    kls_name: String,
    apisets: Vec<APIDataAST>
}

pub enum APIDataAST {
    APIAST(APIAST),
    APISetAST(APISetAST)
}

pub struct Param {
    name: String,
    typ: String
}

pub struct APIAST {
    name: String,
    method: String,
    url: String,
    params: Vec<Param>
}

pub struct APISetAST {
    name: String,
    apisets: Vec<APIDataAST>
}

struct ParserContext {
    parent: Option<Rc<ParserContext>>,
    scope: HashMap<String, String>
}

impl ParserContext {
    fn lookup(&self, key: String) -> Result<String, String> {
        if let Some(val) = self.scope.get(&key) {
            Ok(val.to_string())
        } else {
            match &self.parent {
                Some(s) => s.lookup(key),
                None => Err(format!("{} is not in scope", key))
            }
        }
    }
    pub fn rewrite(&self, expr: String) -> String {
        panic!("Not implemented");
    }
}

pub fn parse(source: RootSchema) -> RootAST {
    panic!("Not implemented");
}