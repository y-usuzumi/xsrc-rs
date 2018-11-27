use std::fmt;
use std::rc::Rc;
use super::super::utils::{Either};

pub struct GenContext{}

pub trait Gen {
    fn gen(&self, _ctx: &GenContext) -> String;
}

#[derive(Debug)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
}

impl Gen for Literal {
    fn gen(&self, _ctx: &GenContext) -> String {
        match self {
            Literal::Number(n) => n.to_string(),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Boolean(b) => b.to_string()
        }
    }
}

#[derive(Debug)]
pub struct Ident(String);

impl Gen for Ident {
    fn gen(&self, _ctx: &GenContext) -> String {
        return self.0.to_owned();
    }
}

#[derive(Debug)]
pub enum DeclType {
    Var,
    Let,
    Const,
}

impl fmt::Display for DeclType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DeclType::Var => "var",
                DeclType::Let => "let",
                DeclType::Const => "const",
            }
        )
    }
}

#[derive(Debug)]
pub struct Decl {
    typ: DeclType,
    ident: Ident
}

impl Gen for Decl {
    fn gen(&self, ctx: &GenContext) -> String {
        format!("{} {}", self.typ, self.ident.gen(ctx))
    }
}

#[derive(Debug)]
pub struct Assign {
    typ: Option<DeclType>,
    ident: Ident,
    expr: Expr
}

impl Gen for Assign {
    fn gen(&self, ctx: &GenContext) -> String {
        match self.typ {
            Some(ref typ) => format!("{} {} = {}", typ, self.ident.gen(ctx), self.expr.gen(ctx)),
            None => format!("{} = {}", self.ident.gen(ctx), self.expr.gen(ctx))
        }
    }
}

#[derive(Debug)]
pub enum Stmt {
    // All expressions are automatically statements
    Expr {
        expr: Expr
    },
    // var a;
    Decl {
        decl: Decl
    },
    // Assign(None, "a", 3) => a = 3;
    // Assign(DeclType::Let, "a", 3) => let a = 3;
    Assign {
        assign: Assign
    },
    // for(let a = 0; a < 10; a++) { ... }
    // TODO: The initialization section of a for-loop supports more than just an assignment
    ForLoop {
        inst: Option<Assign>,
        chk: Option<Expr>,
        incr: Option<Expr>,
        stmts: Vec<Stmt>
    },
}

impl Gen for Stmt {
    fn gen(&self, ctx: &GenContext) -> String {
        match self {
            Stmt::Expr{expr} => format!("{};", expr.gen(ctx)),
            Stmt::Decl{decl} => format!("{};", decl.gen(ctx)),
            Stmt::Assign{assign} => format!("{};", assign.gen(ctx)),
            Stmt::ForLoop{inst, chk, incr, stmts} => {
                let rendered_stmts = stmts.iter().map(|v| v.gen(ctx)).collect::<Vec<String>>().join("\n");
                format!("\
for ({inst}; {chk}; {incr}) {{
{stmts}
}}",
                        inst = inst.as_ref().map_or(String::new(), |ref v| v.gen(ctx)),
                        chk = chk.as_ref().map_or(String::new(), |v| v.gen(ctx)),
                        incr = incr.as_ref().map_or(String::new(), |v| v.gen(ctx)),
                        stmts = rendered_stmts
                )
            }
        }
    }
}

// TODO: Expand this type
#[derive(Debug)]
pub struct CompOp(String);

impl Gen for CompOp {
    fn gen(&self, _ctx: &GenContext) -> String {
        return self.0.to_owned();
    }
}

// TODO: Expand this type
#[derive(Debug)]
pub struct ArithOp(String);

impl Gen for ArithOp {
    fn gen(&self, _ctx: &GenContext) -> String {
        return self.0.to_owned();
    }
}

#[derive(Debug)]
pub enum Expr {
    // 3
    Literal{
        val: Literal
    },
    // a
    Var{
        ident: String
    },
    // a > b
    Comp{
        op: CompOp,
        l: Box<Expr>,
        r: Box<Expr>
    },
    // a + b
    Arith{
        op: ArithOp,
        l: Box<Expr>,
        r: Box<Expr>
    },
    // someFunc("OK", 123)
    FuncCall{
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    // (function (p1, p2) { ... })
    Func{
        ident: String,
        params: Vec<String>,
        stmts: Vec<Stmt>,
        is_async: bool
    },
    // (p1, p2) => { ... }
    ArrowFunc{
        params: Vec<String>,
        body: Either<Vec<Stmt>, Box<Expr>>,
        is_async: bool
    }

}

impl Gen for Expr {
    fn gen(&self, ctx: &GenContext) -> String {
        match self {
            Expr::Literal{ val } => val.gen(ctx),
            Expr::Var{ ident } => ident.to_owned(),
            Expr::Comp{ op, l, r } => format!("({}) {} ({})", l.gen(ctx), op.gen(ctx), r.gen(ctx)),
            Expr::Arith{ op, l, r } => format!("({}) {} ({})", l.gen(ctx), op.gen(ctx), r.gen(ctx)),
            Expr::FuncCall{func, args} => {
                let rendered_args = args.iter().map(|v| v.gen(ctx)).collect::<Vec<String>>().join(", ");
                format!("{}({})", func.gen(ctx), rendered_args)
            },
            Expr::Func{ ident, params, stmts, is_async } => {
                let rendered_stmts = stmts.iter().map(|v| v.gen(ctx)).collect::<Vec<String>>().join("\n");
                format!("\
({async_}function {ident}({params}) {{
{stmts}
}})",
                        async_ = if *is_async { "async " } else { "" },
                        ident = ident,
                        params = params.join(", "),
                        stmts = rendered_stmts
                )
            },
            Expr::ArrowFunc{ params, body, is_async } => {
                let async_ = if *is_async { "async "} else { "" };
                match body {
                    Either::Left(stmts) => {
                        let rendered_stmts = stmts.iter().map(|v| v.gen(ctx)).collect::<Vec<String>>().join("\n");
                        format!("\
({async_}{params}) => {{
{stmts}
}}",
                                async_ = async_,
                                params = params.join(", "),
                                stmts = rendered_stmts)
                    },
                    Either::Right(expr) => {
                        format!("\
                            {async_}({params}) => {expr}",
                                async_ = async_,
                                params = params.join(", "),
                                expr = expr.gen(ctx)
                        )
                    }
                }
            }
        }
    }
}

pub struct Constructor {
    params: Vec<String>,
    stmts: Vec<Stmt>,
}

impl Gen for Constructor {
    fn gen(&self, ctx: &GenContext) -> String {
        let rendered_stmts = self.stmts.iter().map(|v| v.gen(ctx)).collect::<Vec<String>>().join("\n");
        format!("\
            (constructor({params}) {{
{stmts}
}})",
                params = self.params.join(", "),
                stmts = rendered_stmts
        )
    }
}

pub struct Method {
    ident: Ident,
    params: Vec<String>,
    stmts: Vec<Stmt>,
    is_async: bool
}

pub struct Class {
    extends: Rc<Class>,
    constructor: Option<Constructor>,
    methods: Vec<Method>
}


#[cfg(test)]
mod tests {
    mod expr {
        use super::super::*;

        #[test]
        fn const_expr() {
            let ctx = GenContext{};
            let const_expr = Expr::Literal{ val: Literal::String("OK".to_string()) };
            assert_eq!(const_expr.gen(&ctx), "\"OK\"");
        }

        #[test]
        fn var_expr() {
            let ctx = GenContext{};
            let var_expr = Expr::Var{ ident: "someVar".to_string() };
            assert_eq!(var_expr.gen(&ctx), "someVar".to_string());
        }

        #[test]
        fn func_call_expr() {
            let ctx = GenContext{};
            let func_call_expr = Expr::FuncCall{
                // TODO: Improper use of `Var`
                func: box Expr::Var{ ident: "console.log".to_string() },
                args: vec![Expr::Var{ ident: "someVar".to_string() }]
            };
            assert_eq!(func_call_expr.gen(&ctx), "console.log(someVar)");
        }

        #[test]
        fn func_expr() {
            let ctx = GenContext{};
            let stmt = Stmt::Expr{
                expr: Expr::FuncCall{
                    // TODO: Improper use of `Var`
                    func: box Expr::Var{ ident: "console.log".to_string() },
                    args: vec![Expr::Literal{ val: Literal::String("OK".to_string()) }]
                }
            };
            let func_expr = Expr::Func{
                ident: "myFunc".to_string(),
                params: vec!["someVar".to_string()],
                stmts: vec![stmt],
                is_async: false,
            };
            assert_eq!(func_expr.gen(&ctx), "\
(function myFunc(someVar) {
console.log(\"OK\");
})")
        }

        #[test]
        fn arrow_func_expr() {
            let ctx = GenContext{};
            let expr = Expr::FuncCall{
                // TODO: Improper use of `Var`
                func: box Expr::Var{ ident: "console.log".to_string() },
                args: vec![Expr::Literal{ val: Literal::String("OK".to_string()) }]
            };
            let arrow_func_expr = Expr::ArrowFunc{
                params: vec!["someVar".to_string()],
                body: Either::Right(Box::new(expr)),
                is_async: true
            };
            assert_eq!(arrow_func_expr.gen(&ctx), "\
async (someVar) => console.log(\"OK\")\
"
            );
        }
    }

    mod stmt {
        use super::super::*;

        #[test]
        fn expr_stmt() {
            let ctx = GenContext{};
            // TODO: Improper use of `Var` here
            let expr = Expr::FuncCall{
                func: box Expr::Var{ ident: "console.log".to_string() },
                args: vec![Expr::Literal{ val: Literal::String("OK".to_string()) }]
            };
            let expr_stmt = Stmt::Expr{expr};
            assert_eq!(expr_stmt.gen(&ctx), "console.log(\"OK\");")
        }

        #[test]
        fn decl_stmt() {
            let ctx = GenContext{};
            let stmt = Stmt::Decl{
                decl: Decl{typ: DeclType::Const, ident: Ident("hello".to_string())}
            };
            assert_eq!(stmt.gen(&ctx), "const hello;");
        }

        #[test]
        fn assign_stmt() {
            let ctx = GenContext{};
            let func_call = Expr::FuncCall{
                // TODO: Improper use of `Var` here
                func: box Expr::Var{ ident: "console.log".to_string() },
                args: vec![Expr::Literal{ val: Literal::String("OK".to_string()) }]
            };
            let global_assign_stmt = Stmt::Assign{
                assign: Assign{
                    typ: None,
                    ident: Ident("hello".to_string()),
                    expr: func_call,
                }
            };
            assert_eq!(global_assign_stmt.gen(&ctx), "hello = console.log(\"OK\");")
        }

        #[test]
        fn for_loop_stmt() {
            let ctx = GenContext{};
            let func_call_1 = Expr::FuncCall{
                // TODO: Improper use of `Var` here
                func: box Expr::Var{ ident: "console.log".to_string() },
                args: vec![Expr::Literal{ val: Literal::String("OK".to_string()) }]
            };
            let func_args_2 = vec![
                Expr::Arith{
                    op: ArithOp("+".to_string()),
                    l: Box::new(Expr::Literal{ val: Literal::Number(3.0) }),
                    r: Box::new(Expr::Literal{ val: Literal::Number(4.0) }),
                }
            ];
            let func_call_2 = Expr::FuncCall{
                func: Box::new(Expr::Var{ ident: "alert".to_string() }),
                args: func_args_2
            };
            let stmts = vec![
                Stmt::Expr{expr: func_call_1},
                Stmt::Expr{expr: func_call_2},
            ];
            let inst = Some(Assign{
                typ: Some(DeclType::Let),
                ident: Ident("idx".to_string()),
                expr: Expr::Literal{ val: Literal::Number(3.0) }
            });
            let chk = Some(
                Expr::Comp{
                    op: CompOp("<".to_string()),
                    l: Box::new(Expr::Var{ ident: "idx".to_string() }),
                    r: Box::new(Expr::Literal{ val: Literal::Number(10.0) })
                }
            );
            let incr = Some(
                Expr::Comp{
                    op: CompOp("+=".to_string()),
                    l: Box::new(Expr::Var{ ident: "idx".to_string() }),
                    r: Box::new(Expr::Literal{ val: Literal::Number(1.0) })
                }
            );
            let for_loop = Stmt::ForLoop{inst, chk, incr, stmts};
            assert_eq!(for_loop.gen(&ctx), "\
for (let idx = 3; (idx) < (10); (idx) += (1)) {
console.log(\"OK\");
alert((3) + (4));
}\
")
        }
    }

}
