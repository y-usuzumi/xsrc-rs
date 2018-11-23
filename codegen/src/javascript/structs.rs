use std::fmt;
use std::string::ToString;

pub trait Gen {
    fn gen(&self) -> String;
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
        format!("{} {}", self.typ.to_string(), self.ident.to_string())
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
            Some(ref typ) => format!("{} {} = {}", typ.to_string(), self.ident, self.expr.gen()),
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
    Const(String),
    // a
    Var(String),
    // a > b
    Comp(CompOp, Box<Expr<'a>>, Box<Expr<'a>>),
    // a + b
    Arith(ArithOp, Box<Expr<'a>>, Box<Expr<'a>>),
    // someFunc("OK", 123)
    FuncCall(String, &'a [Expr<'a>]),
    // (function (p1, p2) { ... })
    Func(String, &'a [String], &'a [Stmt<'a>]),
    // (p1, p2) => { ... }

}

impl<'a> Gen for Expr<'a> {
    fn gen(&self) -> String {
        match self {
            Expr::Const(s) => s.to_owned(),
            Expr::Var(s) => s.to_owned(),
            Expr::Comp(op, l, r) => format!("({}) {} ({})", l.gen(), op.gen(), r.gen()),
            Expr::Arith(op, l, r) => format!("({}) {} ({})", l.gen(), op.gen(), r.gen()),
            Expr::FuncCall(func, args) => {
                let rendered_args = args.iter().map(|v| v.gen()).collect::<Vec<String>>().join(", ");
                format!("{}({})", func, rendered_args)
            },
            Expr::Func(func, args, stmts) => {
                let rendered_stmts = stmts.iter().map(|v| v.gen()).collect::<Vec<String>>().join("\n");
                format!("\
(function {func}({args}) {{
{stmts}
}})",
                        func = func,
                        args = args.join(", "),
                        stmts = rendered_stmts
                )
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
            let const_expr = Expr::Const("\"OK\"".to_string());
            assert_eq!(const_expr.gen(), "\"OK\"".to_string());
        }

        #[test]
        fn var_expr() {
            let var_expr = Expr::Const("someVar".to_string());
            assert_eq!(var_expr.gen(), "someVar".to_string());
        }

        #[test]
        fn func_call_expr() {
            let args = &[Expr::Var("someVar".to_string())];
            let func_call_expr = Expr::FuncCall(
                "console.log".to_string(),
                args
            );
            assert_eq!(func_call_expr.gen(), "console.log(someVar)");
        }

        #[test]
        fn func_expr() {
            let args = &["someVar".to_string()];
            let func_expr = Expr::Func(
                "console.log".to_string(),
                args,
                &[]
            );
            assert_eq!(func_expr.gen(), "\
(function console.log(someVar) {

})".to_string()
            );
        }
    }

    mod stmt {
        use super::super::*;

        #[test]
        fn expr_stmt() {
            let args = &[Expr::Const("\"OK\"".to_string())];
            let expr = Expr::FuncCall("console.log".to_string(), args);
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
            let func_args = &[Expr::Const("\"OK\"".to_string())];
            let global_assign_stmt = Stmt::Assign{
                assign: Assign{
                    typ: None,
                    ident: "hello".to_string(),
                    expr: Expr::FuncCall("console.log".to_string(), func_args)
                }
            };
            assert_eq!(global_assign_stmt.gen(), "hello = console.log(\"OK\");")
        }

        #[test]
        fn for_loop_stmt() {
            let funcArgs1 = &[Expr::Const("\"OK\"".to_string())];
            let funcArgs2 = &[
                Expr::Arith(
                    ArithOp("+".to_string()),
                    Box::new(Expr::Const("3".to_string())),
                    Box::new(Expr::Const("4".to_string()))
                )
            ];
            let stmts = &[
                Stmt::Expr{expr: Expr::FuncCall("console.log".to_string(), funcArgs1)},
                Stmt::Expr{expr: Expr::FuncCall("alert".to_string(), funcArgs2)},
            ];
            let inst = Some(Assign{
                typ: Some(DeclType::Let),
                ident: "idx".to_string(),
                expr: Expr::Const("0".to_string()),
            });
            let chk = Some(
                Expr::Comp(
                    CompOp("<".to_string()),
                    Box::new(Expr::Var("idx".to_string())),
                    Box::new(Expr::Const("10".to_string())),
                )
            );
            let incr = Some(
                Expr::Arith(
                    ArithOp("+=".to_string()),
                    Box::new(Expr::Var("idx".to_string())),
                    Box::new(Expr::Const("1".to_string())),
                )
            );
            let for_loop = Stmt::ForLoop{inst, chk, incr, stmts};
            assert_eq!(for_loop.gen(), "\
for (let idx = 0; (idx) < (10); (idx) += (1)) {
console.log(\"OK\");
alert((3) + (4));
}\
")
        }
    }

}
