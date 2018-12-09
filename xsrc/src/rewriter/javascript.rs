use crate::transformer::*;
use codegen::javascript::*;
use crate::se_parser as sp;

fn gen_ref(ms: &[sp::Member]) -> Expr {
    let mut expr = Expr::Var("this".to_string());
    for m in ms {
        match m {
            sp::Member::Super => expr = Expr::Member{
                base: box expr,
                member: Ident("_super".to_string())
            },
            sp::Member::Member(m) => expr = Expr::Member{
                base: box expr,
                member: Ident(m.to_string())
            },
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
            sp::Expr::Concat(l, r) => Expr::Arith{
                op: ArithOp("+".to_string()),
                l: box folder(l),
                r: box folder(r)
            }
        }
    }
    match v {
        ContextValue::Expr(expr) => folder(expr)
    }
}

fn root_constructor(root: &ContextBoundedRoot) -> Option<Constructor> {
    if root.params.len() > 0 {
        let stmts = root
            .params
            .iter()
            .map(|p| {
                Stmt::Assign(Assign {
                    typ: None,
                    assignee: Expr::Member{
                        base: box Expr::Var("this".to_string()),
                        member: Ident(p.name.clone())
                    },
                    expr: Expr::Var(p.name.clone())
                })
            })
            .collect::<Vec<Stmt>>();
        Some(Constructor {
            params: root
                .params
                .iter()
                .map(|p| Ident(p.name.clone()))
                .collect::<Vec<Ident>>(),
            stmts
        })
    } else {
        Some(Constructor {
            params: Vec::new(),
            stmts: vec![
                Stmt::Assign(Assign {
                    typ: None,
                    assignee: Expr::Member{
                        base: box Expr::Var("this".to_string()),
                        member: Ident("_url".to_string())
                    },
                    expr: gen_context_value(&root.url)
                })
            ]
        })
    }
}

fn apiset_constructor(apiset: &ContextBoundedAPISet) -> Option<Constructor> {
    let mut stmts = vec![
        Stmt::Assign(Assign {
            typ: None,
            assignee: Expr::Member{
                base: box Expr::Var("this".to_string()),
                member: Ident("_super".to_string())
            },
            expr: Expr::Var("_super".to_string())
        })
    ];
    let mut params = vec![Ident("_super".to_string())];
    if apiset.params.len() > 0 {
        stmts.extend(apiset
            .params
            .iter()
            .map(|p| {
                Stmt::Assign(Assign {
                    typ: None,
                    assignee: Expr::Member{
                        base: box Expr::Var("this".to_string()),
                        member: Ident(p.name.clone())
                    },
                    expr: Expr::Var(p.name.clone())
                })
            }));
        params.extend(
            apiset
                .params
                .iter()
                .map(|p| Ident(p.name.clone()))
                .collect::<Vec<Ident>>()
        );
        Some(Constructor {
            params,
            stmts
        })
    } else {
        stmts.push(Stmt::Assign(Assign {
            typ: None,
            assignee: Expr::Member{
                base: box Expr::Var("this".to_string()),
                member: Ident("_url".to_string())
            },
            expr: gen_context_value(&apiset.url)
        }));
        Some(Constructor {
            params,
            stmts
        })
    }
}

fn gen_apiset(apiset: &ContextBoundedAPISet, code: &mut Code) {
    let kls = Class {
        ident: Ident(apiset.name.to_string()),
        extends: None,
        constructor: apiset_constructor(apiset),
        getters: Vec::new(),
        methods: Vec::new()
    };
    code.stmts.push(Stmt::Class(kls));
}

pub fn gen(root: &ContextBoundedRoot) -> String {
    let root_kls = Class {
        ident: Ident(root.klsname.to_string()),
        extends: None,
        constructor: root_constructor(root),
        getters: Vec::new(),
        methods: Vec::new(),
    };
    let code = Code {
        stmts: vec![Stmt::Class(root_kls)],
    };
    code.gen(&GenContext {})
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
