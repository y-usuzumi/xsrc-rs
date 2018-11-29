use std::rc::Rc;
use std::iter::FromIterator;
use std::collections::HashMap;
use super::schema::{RootSchema, APIDataMap, APIData};

pub struct RootAST {
    url: Option<String>,
    klsname: String,
    apisets: HashMap<String, APIDataAST>
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
    apisets: HashMap<String, APIDataAST>
}

struct ParserContext {
    parent: Option<Rc<ParserContext>>,
    scope: HashMap<String, String>
}

impl ParserContext {
    fn build(parent: Option<Rc<ParserContext>>) -> ParserContext {
        panic!("Not implemented")
    }
    fn lookup(&self, key: &str) -> Result<String, String> {
        if let Some(val) = self.scope.get(key) {
            Ok(val.to_string())
        } else {
            match &self.parent {
                Some(s) => s.lookup(key),
                None => Err(format!("{} is not in scope", key))
            }
        }
    }
    pub fn rewrite(&self, expr: &str) -> String {
        panic!("Not implemented");
    }
}

fn parse_apiset(name: &str, apiset: &APIData) -> APIDataAST {
    match apiset {
        APIData::APISet(schema) => {
            let children = HashMap::from_iter(
                schema.apisets.iter().map(|(k, v)| (k.to_string(), parse_apiset(k, v)))
            );
            APIDataAST::APISetAST(APISetAST{
                name: name.to_string(),
                apisets: children
            })
        },
        APIData::API(schema) => {
            APIDataAST::APIAST(APIAST{
                name: name.to_string(),
                method: schema.method.to_string(),
                url: schema.url.to_string(),
                params: Vec::new()
            })
        }
    }
}

pub fn parse(source: RootSchema) -> RootAST {
    RootAST {
        url: source.url,
        klsname: source.klsname,
        apisets: HashMap::from_iter((*source.apisets).iter().map(|(k, v)| (k.to_string(), parse_apiset(k, v))))
    }
}