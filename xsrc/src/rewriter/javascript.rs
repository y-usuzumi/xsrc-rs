use crate::se_parser as sp;
use crate::transformer::*;
use codegen::javascript::*;
use linked_hash_map::LinkedHashMap;
use std::collections::HashMap;
use std::iter::FromIterator;
use utils::linked_hashmap;

fn gen_ref(ms: &[sp::Member]) -> Expr {
    let mut expr = Expr::Var("this".to_string());
    for m in ms {
        match m {
            sp::Member::Super => {
                expr = Expr::Member {
                    base: box expr,
                    member: Ident("_super".to_string()),
                }
            }
            sp::Member::Member(m) => {
                expr = Expr::Member {
                    base: box expr,
                    member: Ident(m.to_string()),
                }
            }
        }
    }
    expr
}

fn gen_context_value(v: &ContextValue) -> Expr {
    fn folder(expr: &sp::Expr) -> Expr {
        match expr {
            sp::Expr::Lit(s) => Expr::Literal(Literal::String(s.to_string())),
            sp::Expr::Ref(r) => gen_ref(r),
            sp::Expr::Var(s) => Expr::Var(s.to_string()),
            sp::Expr::Concat(l, r) => Expr::Arith {
                op: ArithOp("+".to_string()),
                l: box folder(l),
                r: box folder(r),
            },
        }
    }
    match v {
        ContextValue::Expr(expr) => folder(expr),
    }
}

fn root_constructor(root: &ContextBoundedRoot) -> Option<Constructor> {
    if root.bounded_vars.len() > 0 {
        let stmts = root
            .bounded_vars
            .iter()
            .map(|(_, p)| {
                Stmt::Assign(Assign {
                    typ: None,
                    assignee: Expr::Member {
                        base: box Expr::Var("this".to_string()),
                        member: Ident(format!("_{}", p.name.clone())),
                    },
                    expr: Expr::Var(p.name.clone()),
                })
            })
            .collect::<Vec<Stmt>>();
        Some(Constructor {
            params: root
                .bounded_vars
                .iter()
                .map(|(_, p)| Ident(p.name.clone()))
                .collect::<Vec<Ident>>(),
            stmts,
        })
    } else {
        Some(Constructor {
            params: Vec::new(),
            stmts: vec![Stmt::Assign(Assign {
                typ: None,
                assignee: Expr::Member {
                    base: box Expr::Var("this".to_string()),
                    member: Ident("_url".to_string()),
                },
                expr: gen_context_value(&root.url),
            })],
        })
    }
}

fn apiset_constructor(apiset: &ContextBoundedAPISet) -> Option<Constructor> {
    let mut stmts = vec![
        Stmt::Assign(Assign {
            typ: None,
            assignee: Expr::Member {
                base: box Expr::Var("this".to_string()),
                member: Ident("_super".to_string()),
            },
            expr: Expr::Var("_super".to_string()),
        }),
        Stmt::Assign(Assign {
            typ: None,
            assignee: Expr::Member {
                base: box Expr::Var("this".to_string()),
                member: Ident("_url".to_string()),
            },
            expr: gen_context_value(&apiset.url),
        }),
    ];
    let mut params = vec![Ident("_super".to_string())];
    stmts.extend(apiset.bounded_vars.iter().map(|(_, p)| {
        Stmt::Assign(Assign {
            typ: None,
            assignee: Expr::Member {
                base: box Expr::Var("this".to_string()),
                member: Ident(p.name.clone()),
            },
            expr: Expr::Var(p.name.clone()),
        })
    }));
    params.extend(
        apiset
            .bounded_vars
            .iter()
            .map(|(_, p)| Ident(p.name.clone()))
            .collect::<Vec<Ident>>(),
    );
    Some(Constructor { params, stmts })
}

fn gen_apiset(apiset: &ContextBoundedAPISet, code: &mut Code, parent_kls: &mut Class) {
    let mut kls = Class {
        ident: Ident(apiset.name.to_string()),
        extends: None,
        constructor: apiset_constructor(apiset),
        getters: Vec::new(),
        methods: Vec::new(),
    };
    for (k, child) in &apiset.apisets {
        match child {
            ContextBoundedAPIData::API(child) => {
                gen_api(child, &mut kls);
            }
            ContextBoundedAPIData::APISet(child) => {
                gen_apiset(&child, code, &mut kls);
                kls.getters.push(Getter {
                    ident: Ident(k.to_string()),
                    stmts: vec![Stmt::Return(Expr::Instantiate {
                        constructor: box Expr::Var(k.to_string()),
                        args: vec![Expr::Var("this".to_string())],
                    })],
                })
            }
        }
    }
    // This must come at the end because Vec will take ownership of kls
    code.stmts.push(Stmt::Class(kls));
}

fn gen_axios_call(
    url: &ContextValue,
    method: &HttpMethod,
    params: &LinkedHashMap<String, Param>,
    data: &LinkedHashMap<String, Param>,
) -> Expr {
    let url_expr = gen_context_value(url);
    let method = match method {
        HttpMethod::GET => "get",
        HttpMethod::POST => "post",
        HttpMethod::PUT => "put",
        HttpMethod::DELETE => "delete",
        HttpMethod::HEAD => "head",
        HttpMethod::OPTIONS => "options",
        HttpMethod::PATCH => "patch",
    }
    .to_string();
    let mut axios_config = linked_hashmap! {
        "method".to_string() => Expr::Literal(Literal::String(method)),
        "url".to_string() => url_expr,
    };
    if params.len() > 0 {
        let params = LinkedHashMap::from_iter(
            params
                .iter()
                .map(|(k, v)| (k.to_string(), Expr::Var(v.name.to_string()))),
        );
        axios_config.insert("params".to_string(), Expr::Object(params));
    }
    if data.len() > 0 {
        let data = LinkedHashMap::from_iter(
            data.iter()
                .map(|(k, v)| (k.to_string(), Expr::Var(v.name.to_string()))),
        );
        axios_config.insert("data".to_string(), Expr::Object(data));
    }
    let args = vec![Expr::Object(axios_config)];
    Expr::FuncCall {
        func: box Expr::Var("axios".to_string()),
        args,
    }
}

fn gen_api(api: &ContextBoundedAPI, kls: &mut Class) {
    let stmts = vec![Stmt::Return(gen_axios_call(
        &api.url,
        &api.method,
        &api.params,
        &api.data,
    ))];
    let method = Method {
        ident: Ident(api.name.to_string()),
        params: api
            .bounded_vars
            .iter()
            .map(|(_, p)| p.name.to_string())
            .collect::<Vec<String>>(),
        stmts,
        is_async: true,
    };
    kls.methods.push(method);
}

fn gen_root(root: &ContextBoundedRoot, code: &mut Code) {
    let mut root_kls = Class {
        ident: Ident(root.klsname.to_string()),
        extends: None,
        constructor: root_constructor(root),
        getters: vec![Getter {
            ident: Ident("url".to_string()),
            stmts: vec![Stmt::Return(Expr::Member {
                base: box Expr::Var("this".to_string()),
                member: Ident("_url".to_string()),
            })],
        }],
        methods: Vec::new(),
    };
    for (k, child) in &root.apisets {
        match child {
            ContextBoundedAPIData::API(child) => gen_api(&child, &mut root_kls),
            ContextBoundedAPIData::APISet(child) => {
                gen_apiset(&child, code, &mut root_kls);
                root_kls.getters.push(Getter {
                    ident: Ident(k.to_string()),
                    stmts: vec![Stmt::Return(Expr::Instantiate {
                        constructor: box Expr::Var(k.to_string()),
                        args: vec![Expr::Var("this".to_string())],
                    })],
                })
            }
        }
    }
    code.stmts.push(Stmt::Export {
        stmt: box Stmt::Class(root_kls),
        is_default: true,
    });
}

pub fn gen(root: &ContextBoundedRoot, gen_ctx: &GenContext) -> String {
    let stmts = vec![Stmt::Import(Import {
        def: Some(Ident("axios".to_string())),
        imps: None,
        path: "axios".to_string(),
    })];
    let mut code = Code { stmts };
    gen_root(root, &mut code);
    code.gen(gen_ctx)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gen() {
        // let root = ContextBoundedRoot {
        //     klsname: "XiaoSi".to_string(),
        //     url: ContextValue::Expr(sp::Expr::Lit(""))
        // };
        println!("===== TODO =====");
    }
}
