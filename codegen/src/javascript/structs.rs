use std::fmt;
use std::string::ToString;

pub trait Gen {
    fn gen(&self) -> String;
}

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

pub enum Stmt<'a> {
    // var a;
    Decl(DeclType, String),
    // Assign(None, "a", 3) => a = 3;
    // Assign(DeclType::Let, "a", 3) => let a = 3;
    Assign(Option<DeclType>, String, Expr<'a>),
}

impl<'a> Gen for Stmt<'a> {
    fn gen(&self) -> String {
        match self {
            Stmt::Decl(t, s) => format!("{} {};", t.to_string(), s),
            Stmt::Assign(None, s, expr) => format!("{} = {};", s, expr.gen()),
            Stmt::Assign(Some(t), s, expr) => format!("{} {} = {};", t.to_string(), s, expr.gen()),
        }
    }
}

pub enum Expr<'a> {
    // 3
    Const(String),
    // a
    Var(String),
    // someFunc("OK", 123)
    FuncCall(String, &'a [String]),
    // (function (p1, p2) { ... })
    Func(String, &'a [String], &'a [Stmt<'a>]),
    // (p1, p2) => { ... }
}

impl<'a> Gen for Expr<'a> {
    fn gen(&self) -> String {
        match self {
            Expr::Const(s) => s.to_owned(),
            Expr::Var(s) => s.to_owned(),
            Expr::FuncCall(func, args) => format!("{}({})", func, args.join(", ")),
            Expr::Func(func, args, _stmts) => format!("\
(function {func}({args}){{
    {stmts}
}}",
                func = func,
                args = args.join(", "),
                stmts = ";"
            ),
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
            let args = &["someVar".to_string()];
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
function console.log(someVar) {

}".to_string()
            );
        }
    }

}
