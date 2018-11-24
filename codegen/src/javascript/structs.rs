use std::fmt;
use super::super::utils::{Either};

pub trait Gen {
    fn gen(&self) -> String;
}

#[derive(Debug)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
}

impl Gen for Literal {
    fn gen(&self) -> String {
        match self {
            Literal::Number(n) => n.to_string(),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Boolean(b) => b.to_string()
        }
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
    ident: String
}

impl Gen for Decl {
    fn gen(&self) -> String {
        format!("{} {}", self.typ, self.ident)
    }
}

#[derive(Debug)]
pub struct Assign<'a>{
    typ: Option<DeclType>,
    ident: String,
    expr: Expr<'a>
}

impl<'a> Gen for Assign<'a> {
    fn gen(&self) -> String {
        match self.typ {
            Some(ref typ) => format!("{} {} = {}", typ, self.ident, self.expr.gen()),
            None => format!("{} = {}", self.ident, self.expr.gen())
        }
    }
}

#[derive(Debug)]
pub enum Stmt<'a> {
    // All expressions are automatically statements
    Expr {
        expr: Expr<'a>
    },
    // var a;
    Decl {
        decl: Decl
    },
    // Assign(None, "a", 3) => a = 3;
    // Assign(DeclType::Let, "a", 3) => let a = 3;
    Assign {
        assign: Assign<'a>
    },
    // for(let a = 0; a < 10; a++) { ... }
    // TODO: The initialization section of a for-loop supports more than just an assignment
    ForLoop {
        inst: Option<Assign<'a>>,
        chk: Option<Expr<'a>>,
        incr: Option<Expr<'a>>,
        stmts: &'a [Stmt<'a>]
    },
}

impl<'a> Gen for Stmt<'a> {
    fn gen(&self) -> String {
        match self {
            Stmt::Expr{expr} => format!("{};", expr.gen()),
            Stmt::Decl{decl} => format!("{};", decl.gen()),
            Stmt::Assign{assign} => format!("{};", assign.gen()),
            Stmt::ForLoop{inst, chk, incr, stmts} => {
                let rendered_stmts = stmts.iter().map(|v| v.gen()).collect::<Vec<String>>().join("\n");
                format!("\
for ({inst}; {chk}; {incr}) {{
{stmts}
}}",
                        inst = inst.as_ref().map_or(String::new(), |ref v| v.gen()),
                        chk = chk.as_ref().map_or(String::new(), |v| v.gen()),
                        incr = incr.as_ref().map_or(String::new(), |v| v.gen()),
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
    fn gen(&self) -> String {
        return self.0.to_owned();
    }
}

// TODO: Expand this type
#[derive(Debug)]
pub struct ArithOp(String);

impl Gen for ArithOp {
    fn gen(&self) -> String {
        return self.0.to_owned();
    }
}

#[derive(Debug)]
pub enum Expr<'a> {
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
        l: Box<Expr<'a>>,
        r: Box<Expr<'a>>
    },
    // a + b
    Arith{
        op: ArithOp,
        l: Box<Expr<'a>>,
        r: Box<Expr<'a>>
    },
    // someFunc("OK", 123)
    FuncCall{
        func: Box<Expr<'a>>,
        args: &'a [Expr<'a>]
    },
    // (function (p1, p2) { ... })
    Func{
        ident: String,
        params: &'a [String],
        stmts: &'a [Stmt<'a>]
    },
    // (p1, p2) => { ... }
    ArrowFunc{
        params: &'a [String],
        body: Either<&'a [Stmt<'a>], Box<Expr<'a>>>
    }

}

impl<'a> Gen for Expr<'a> {
    fn gen(&self) -> String {
        match self {
            Expr::Literal{ val } => val.gen(),
            Expr::Var{ ident } => ident.to_owned(),
            Expr::Comp{ op, l, r } => format!("({}) {} ({})", l.gen(), op.gen(), r.gen()),
            Expr::Arith{ op, l, r } => format!("({}) {} ({})", l.gen(), op.gen(), r.gen()),
            Expr::FuncCall{func, args} => {
                let rendered_args = args.iter().map(|v| v.gen()).collect::<Vec<String>>().join(", ");
                format!("{}({})", func.gen(), rendered_args)
            },
            Expr::Func{ ident, params, stmts} => {
                let rendered_stmts = stmts.iter().map(|v| v.gen()).collect::<Vec<String>>().join("\n");
                format!("\
(function {ident}({params}) {{
{stmts}
}})",
                        ident = ident,
                        params = params.join(", "),
                        stmts = rendered_stmts
                )
            },
            Expr::ArrowFunc{ params, body } => {
                match body {
                    Either::Left(stmts) => {
                        let rendered_stmts = stmts.iter().map(|v| v.gen()).collect::<Vec<String>>().join("\n");
                        format!("\
({params}) => {{
{stmts}
}}",
                                params = params.join(", "),
                                stmts = rendered_stmts)
                    },
                    Either::Right(expr) => {
                        format!("\
                            ({params}) => {expr}",
                                params = params.join(", "),
                                expr = expr.gen()
                        )
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    mod expr {
        use super::super::*;

        #[test]
        fn const_expr() {
            let const_expr = Expr::Literal{ val: Literal::String("OK".to_string()) };
            assert_eq!(const_expr.gen(), "\"OK\"");
        }

        #[test]
        fn var_expr() {
            let var_expr = Expr::Var{ ident: "someVar".to_string() };
            assert_eq!(var_expr.gen(), "someVar".to_string());
        }

        #[test]
        fn func_call_expr() {
            let args = &[Expr::Var{ ident: "someVar".to_string() }];
            let func_call_expr = Expr::FuncCall{
                // TODO: Improper use of `Var`
                func: Box::new(Expr::Var{ ident: "console.log".to_string() }),
                args: args
            };
            assert_eq!(func_call_expr.gen(), "console.log(someVar)");
        }

        #[test]
        fn func_expr() {
            let params = &["someVar".to_string()];
            let stmt = Stmt::Expr{
                expr: Expr::FuncCall{
                    // TODO: Improper use of `Var`
                    func: Box::new(Expr::Var{ ident: "console.log".to_string() }),
                    args: &[Expr::Literal{ val: Literal::String("OK".to_string()) }]
                }
            };
            let func_expr = Expr::Func{
                ident: "myFunc".to_string(),
                params,
                stmts: &[stmt]
            };
            assert_eq!(func_expr.gen(), "\
(function myFunc(someVar) {
console.log(\"OK\");
})".to_string()
            );
        }

        #[test]
        fn arrow_func_expr() {
            let params = &["someVar".to_string()];
            let expr = Expr::FuncCall{
                // TODO: Improper use of `Var`
                func: Box::new(Expr::Var{ ident: "console.log".to_string() }),
                args: &[Expr::Literal{ val: Literal::String("OK".to_string()) }]
            };
            let arrow_func_expr = Expr::ArrowFunc{
                params,
                body: Either::Right(Box::new(expr))
            };
            assert_eq!(arrow_func_expr.gen(), "\
(someVar) => console.log(\"OK\")\
"
            );
        }
    }

    mod stmt {
        use super::super::*;

        #[test]
        fn expr_stmt() {
            let args = &[Expr::Literal{ val: Literal::String("OK".to_string()) }];
            // TODO: Improper use of `Var` here
            let expr = Expr::FuncCall{
                func: Box::new(Expr::Var{ ident: "console.log".to_string() }),
                args: args
            };
            let expr_stmt = Stmt::Expr{expr};
            assert_eq!(expr_stmt.gen(), "console.log(\"OK\");")
        }

        #[test]
        fn decl_stmt() {
            let stmt = Stmt::Decl{decl: Decl{typ: DeclType::Const, ident: "hello".to_string()}};
            assert_eq!(stmt.gen(), "const hello;");
        }

        #[test]
        fn assign_stmt() {
            let args = &[Expr::Literal{ val: Literal::String("OK".to_string()) }];
            let func_call = Expr::FuncCall{
                // TODO: Improper use of `Var` here
                func: Box::new(Expr::Var{ ident: "console.log".to_string() }),
                args: args
            };
            let global_assign_stmt = Stmt::Assign{
                assign: Assign{
                    typ: None,
                    ident: "hello".to_string(),
                    expr: func_call,
                }
            };
            assert_eq!(global_assign_stmt.gen(), "hello = console.log(\"OK\");")
        }

        #[test]
        fn for_loop_stmt() {
            let func_args_1 = &[Expr::Literal{ val: Literal::String("OK".to_string()) }];
            let func_call_1 = Expr::FuncCall{
                // TODO: Improper use of `Var` here
                func: Box::new(Expr::Var{ ident: "console.log".to_string() }),
                args: func_args_1
            };
            let func_args_2 = &[
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
            let stmts = &[
                Stmt::Expr{expr: func_call_1},
                Stmt::Expr{expr: func_call_2},
            ];
            let inst = Some(Assign{
                typ: Some(DeclType::Let),
                ident: "idx".to_string(),
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
            assert_eq!(for_loop.gen(), "\
for (let idx = 3; (idx) < (10); (idx) += (1)) {
console.log(\"OK\");
alert((3) + (4));
}\
")
        }
    }

}
