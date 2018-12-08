use crate::transformer::*;
use codegen::javascript::*;
use crate::se_parser as sp;

fn root_constructor(root: &ContextBoundedRoot) -> Option<Constructor> {
    if root.params.len() > 0 {
        let stmts = root
            .params
            .iter()
            .map(|p| {
                Stmt::Assign(Assign {
                    typ: None,
                    ident: Ident(format!("this._{}", p.name)),
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
        None
    }
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
        panic!("TODO");
    }
}
