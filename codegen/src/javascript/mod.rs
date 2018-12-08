use super::utils::Either;
use super::utils::Either::*;
use std::fmt;

pub struct GenContext {}

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
            Literal::Boolean(b) => b.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Ident(pub String);

impl Gen for Ident {
    fn gen(&self, _ctx: &GenContext) -> String {
        return self.0.to_string();
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
    ident: Ident,
}

impl Gen for Decl {
    fn gen(&self, ctx: &GenContext) -> String {
        format!("{} {}", self.typ, self.ident.gen(ctx))
    }
}

#[derive(Debug)]
pub struct Assign {
    pub typ: Option<DeclType>,
    pub ident: Ident,
    pub expr: Expr,
}

impl Gen for Assign {
    fn gen(&self, ctx: &GenContext) -> String {
        match self.typ {
            Some(ref typ) => format!("{} {} = {}", typ, self.ident.gen(ctx), self.expr.gen(ctx)),
            None => format!("{} = {}", self.ident.gen(ctx), self.expr.gen(ctx)),
        }
    }
}

#[derive(Debug)]
pub enum Stmt {
    // All expressions are automatically statements
    Expr(Expr),
    // var a;
    Decl(Decl),
    // Assign(None, "a", 3) => a = 3;
    // Assign(DeclType::Let, "a", 3) => let a = 3;
    Assign(Assign),
    Return(Expr),
    // for(let a = 0; a < 10; a++) { ... }
    // TODO: The initialization section of a for-loop supports more than just an assignment
    ForLoop {
        inst: Option<Assign>,
        chk: Option<Expr>,
        incr: Option<Expr>,
        stmts: Vec<Stmt>,
    },
    Import(Import),
    Class(Class),
    Export {
        is_default: bool,
        stmt: Box<Stmt>
    },
}

impl Gen for Stmt {
    fn gen(&self, ctx: &GenContext) -> String {
        match self {
            Stmt::Expr(expr) => format!("{};", expr.gen(ctx)),
            Stmt::Decl(decl) => format!("{};", decl.gen(ctx)),
            Stmt::Assign(assign) => format!("{};", assign.gen(ctx)),
            Stmt::ForLoop {
                inst,
                chk,
                incr,
                stmts,
            } => {
                let rendered_stmts = stmts
                    .iter()
                    .map(|v| v.gen(ctx))
                    .collect::<Vec<String>>()
                    .join("\n");
                format!(
                    "\
for ({inst}; {chk}; {incr}) {{
{stmts}
}}",
                    inst = inst.as_ref().map_or(String::new(), |ref v| v.gen(ctx)),
                    chk = chk.as_ref().map_or(String::new(), |v| v.gen(ctx)),
                    incr = incr.as_ref().map_or(String::new(), |v| v.gen(ctx)),
                    stmts = rendered_stmts
                )
            }
            Stmt::Return(expr) => format!("return {};", expr.gen(ctx)),
            Stmt::Import(imp) => imp.gen(ctx),
            Stmt::Export { is_default, stmt } => {
                if *is_default {
                    format!("export default {}", stmt.gen(ctx))
                } else {
                    format!("export {}", stmt.gen(ctx))
                }
            },
            Stmt::Class(kls) => {
                kls.gen(ctx)
            }
        }
    }
}

// TODO: Expand this type
#[derive(Debug)]
pub struct CompOp(String);

impl Gen for CompOp {
    fn gen(&self, _ctx: &GenContext) -> String {
        return self.0.to_string();
    }
}

// TODO: Expand this type
#[derive(Debug)]
pub struct ArithOp(String);

impl Gen for ArithOp {
    fn gen(&self, _ctx: &GenContext) -> String {
        return self.0.to_string();
    }
}

#[derive(Debug)]
pub enum Expr {
    // 3
    Literal(Literal),
    // a
    Var(String),
    // a > b
    Comp {
        op: CompOp,
        l: Box<Expr>,
        r: Box<Expr>,
    },
    // a + b
    Arith {
        op: ArithOp,
        l: Box<Expr>,
        r: Box<Expr>,
    },
    // someFunc("OK", 123)
    FuncCall {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    // (function (p1, p2) { ... })
    Func {
        ident: String,
        params: Vec<String>,
        stmts: Vec<Stmt>,
        is_async: bool,
    },
    // (p1, p2) => { ... }
    ArrowFunc {
        params: Vec<String>,
        body: Either<Vec<Stmt>, Box<Expr>>,
        is_async: bool,
    },
}

impl Gen for Expr {
    fn gen(&self, ctx: &GenContext) -> String {
        match self {
            Expr::Literal(val) => val.gen(ctx),
            Expr::Var(ident) => ident.to_string(),
            Expr::Comp { op, l, r } => format!("({}) {} ({})", l.gen(ctx), op.gen(ctx), r.gen(ctx)),
            Expr::Arith { op, l, r } => {
                format!("({}) {} ({})", l.gen(ctx), op.gen(ctx), r.gen(ctx))
            }
            Expr::FuncCall { func, args } => {
                let rendered_args = args
                    .iter()
                    .map(|v| v.gen(ctx))
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("{}({})", func.gen(ctx), rendered_args)
            }
            Expr::Func {
                ident,
                params,
                stmts,
                is_async,
            } => {
                let rendered_stmts = stmts
                    .iter()
                    .map(|v| v.gen(ctx))
                    .collect::<Vec<String>>()
                    .join("\n");
                format!(
                    "\
({async_}function {ident}({params}) {{
{stmts}
}})",
                    async_ = if *is_async { "async " } else { "" },
                    ident = ident,
                    params = params.join(", "),
                    stmts = rendered_stmts
                )
            }
            Expr::ArrowFunc {
                params,
                body,
                is_async,
            } => {
                let async_ = if *is_async { "async " } else { "" };
                match body {
                    Left(stmts) => {
                        let rendered_stmts = stmts
                            .iter()
                            .map(|v| v.gen(ctx))
                            .collect::<Vec<String>>()
                            .join("\n");
                        format!(
                            "\
({async_}{params}) => {{
{stmts}
}}",
                            async_ = async_,
                            params = params.join(", "),
                            stmts = rendered_stmts
                        )
                    }
                    Right(expr) => format!(
                        "\
                         {async_}({params}) => {expr}",
                        async_ = async_,
                        params = params.join(", "),
                        expr = expr.gen(ctx)
                    ),
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Constructor {
    pub params: Vec<Ident>,
    pub stmts: Vec<Stmt>,
}

impl Gen for Constructor {
    fn gen(&self, ctx: &GenContext) -> String {
        let rendered_stmts = self
            .stmts
            .iter()
            .map(|v| v.gen(ctx))
            .collect::<Vec<String>>()
            .join("\n");
        format!(
            "\
constructor({params}) {{
{stmts}
}}",
            params = self
                .params
                .iter()
                .map(|v| v.gen(ctx))
                .collect::<Vec<String>>()
                .join(", "),
            stmts = rendered_stmts
        )
    }
}

#[derive(Debug)]
pub struct Method {
    ident: Ident,
    params: Vec<String>,
    stmts: Vec<Stmt>,
    is_async: bool,
}

impl Gen for Method {
    fn gen(&self, ctx: &GenContext) -> String {
        let rendered_stmts = self
            .stmts
            .iter()
            .map(|v| v.gen(ctx))
            .collect::<Vec<String>>()
            .join("\n");
        format!(
            "\
{async_}{ident}({params}) {{
{stmts}
}}",
            async_ = if self.is_async { "async " } else { "" },
            ident = self.ident.gen(ctx),
            params = self.params.join(", "),
            stmts = rendered_stmts
        )
    }
}

#[derive(Debug)]
pub struct Getter {
    ident: Ident,
    stmts: Vec<Stmt>,
}

impl Gen for Getter {
    fn gen(&self, ctx: &GenContext) -> String {
        let rendered_stmts = self
            .stmts
            .iter()
            .map(|v| v.gen(ctx))
            .collect::<Vec<String>>()
            .join("\n");
        format!(
            "\
get {ident}() {{
{stmts}
}}",
            ident = self.ident.gen(ctx),
            stmts = rendered_stmts
        )
    }
}

#[derive(Debug)]
pub struct Class {
    pub ident: Ident,
    pub extends: Option<Ident>,
    pub constructor: Option<Constructor>,
    pub methods: Vec<Method>,
    pub getters: Vec<Getter>
}

impl Gen for Class {
    fn gen(&self, ctx: &GenContext) -> String {
        let mut rendered_decls: Vec<String> = Vec::new();
        match &self.constructor {
            Some(c) => {
                rendered_decls.push(c.gen(ctx));
            }
            None => {}
        };
        rendered_decls.extend(self.methods.iter().map(|v| v.gen(ctx)));
        rendered_decls.extend(self.getters.iter().map(|v| v.gen(ctx)));
        format!(
            "\
class {ident} {extends}{{
{decls}}}",
            ident = self.ident.gen(ctx),
            extends = match &self.extends {
                Some(c) => format!("extends {} ", c.gen(ctx)),
                None => String::new(),
            },
            decls = rendered_decls.iter().map(|decl| format!("{}\n", decl)).collect::<Vec<String>>().join(""),
        )
    }
}

#[derive(Debug)]
pub enum ImportName {
    Simple(Ident),
    Alias(Ident, Ident),
}

impl Gen for ImportName {
    fn gen(&self, ctx: &GenContext) -> String {
        match self {
            ImportName::Simple(ident) => ident.gen(ctx),
            ImportName::Alias(old, new) => format!("{} as {}", old.gen(ctx), new.gen(ctx)),
        }
    }
}

#[derive(Debug)]
pub struct ImportStar();

#[derive(Debug)]
pub struct Import {
    def: Option<Ident>,
    imps: Option<Either<ImportStar, Vec<ImportName>>>,
    path: String,
}

impl Gen for Import {
    fn gen(&self, ctx: &GenContext) -> String {
        if self.def.is_some() || self.imps.is_some() {
            let mut all_imps = Vec::new();
            let mut named_imps = Vec::new();
            match &self.def {
                Some(ident) => all_imps.push(ident.gen(ctx)),
                None => {}
            }
            match &self.imps {
                Some(Left(_)) => all_imps.push("*".to_string()),
                Some(Right(names)) => named_imps.extend(names.iter().map(|v| v.gen(ctx))),
                None => {}
            }
            if named_imps.len() > 0 {
                let rendered_named_imps = format!("{{{imps}}}", imps = named_imps.join(", "));
                all_imps.push(rendered_named_imps);
            }
            format!(
                "import {imps} from \"{path}\";",
                imps = all_imps.join(", "),
                path = self.path
            )
        } else {
            format!("import \"{path}\";", path = self.path)
        }
    }
}

pub struct Code {
    pub stmts: Vec<Stmt>
}

impl Gen for Code {
    fn gen(&self, ctx: &GenContext) -> String {
        let mut s = String::new();
        for stmt in &self.stmts {
            s.push_str(&format!("{}\n", stmt.gen(ctx)));
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn const_expr() {
        let ctx = GenContext {};
        let const_expr = Expr::Literal(Literal::String("OK".to_string()));
        assert_eq!(const_expr.gen(&ctx), "\"OK\"");
    }

    #[test]
    fn var_expr() {
        let ctx = GenContext {};
        let var_expr = Expr::Var("someVar".to_string());
        assert_eq!(var_expr.gen(&ctx), "someVar".to_string());
    }

    #[test]
    fn func_call_expr() {
        let ctx = GenContext {};
        let func_call_expr = Expr::FuncCall {
            // TODO: Improper use of `Var`
            func: box Expr::Var("console.log".to_string()),
            args: vec![Expr::Var("someVar".to_string())]
        };
        assert_eq!(func_call_expr.gen(&ctx), "console.log(someVar)");
    }

    #[test]
    fn func_expr() {
        let ctx = GenContext {};
        let stmt = Stmt::Expr(
            Expr::FuncCall {
                // TODO: Improper use of `Var`
                func: box Expr::Var("console.log".to_string()),
                args: vec![Expr::Literal(Literal::String("OK".to_string()))]
            }
        );
        let func_expr = Expr::Func {
            ident: "myFunc".to_string(),
            params: vec!["someVar".to_string()],
            stmts: vec![stmt],
            is_async: false,
        };
        assert_eq!(
            func_expr.gen(&ctx),
            "\
(function myFunc(someVar) {
console.log(\"OK\");
})"
        )
    }

    #[test]
    fn arrow_func_expr() {
        let ctx = GenContext {};
        let expr = Expr::FuncCall {
            // TODO: Improper use of `Var`
            func: box Expr::Var("console.log".to_string()),
            args: vec![Expr::Literal(Literal::String("OK".to_string()))]
        };
        let arrow_func_expr = Expr::ArrowFunc {
            params: vec!["someVar".to_string()],
            body: Either::Right(box expr),
            is_async: true,
        };
        assert_eq!(
            arrow_func_expr.gen(&ctx),
            "\
             async (someVar) => console.log(\"OK\")\
             "
        );
    }

    #[test]
    fn expr_stmt() {
        let ctx = GenContext {};
        // TODO: Improper use of `Var` here
        let expr = Expr::FuncCall {
            func: box Expr::Var("console.log".to_string()),
            args: vec![Expr::Literal(Literal::String("OK".to_string()))]
        };
        let expr_stmt = Stmt::Expr(expr);
        assert_eq!(expr_stmt.gen(&ctx), "console.log(\"OK\");")
    }

    #[test]
    fn decl_stmt() {
        let ctx = GenContext {};
        let stmt = Stmt::Decl(
            Decl {
                typ: DeclType::Const,
                ident: Ident("hello".to_string()),
            }
        );
        assert_eq!(stmt.gen(&ctx), "const hello;");
    }

    #[test]
    fn assign_stmt() {
        let ctx = GenContext {};
        let func_call = Expr::FuncCall {
            // TODO: Improper use of `Var` here
            func: box Expr::Var("console.log".to_string()),
            args: vec![Expr::Literal(Literal::String("OK".to_string()))]
        };
        let global_assign_stmt = Stmt::Assign(
            Assign {
                typ: None,
                ident: Ident("hello".to_string()),
                expr: func_call,
            }
        );
        assert_eq!(global_assign_stmt.gen(&ctx), "hello = console.log(\"OK\");")
    }

    #[test]
    fn for_loop_stmt() {
        let ctx = GenContext {};
        let func_call_1 = Expr::FuncCall {
            // TODO: Improper use of `Var` here
            func: box Expr::Var("console.log".to_string()),
            args: vec![Expr::Literal(Literal::String("OK".to_string()))]
        };
        let func_args_2 = vec![Expr::Arith {
            op: ArithOp("+".to_string()),
            l: box Expr::Literal(Literal::Number(3.0)),
            r: box Expr::Literal(Literal::Number(4.0))
        }];
        let func_call_2 = Expr::FuncCall {
            func: box Expr::Var("alert".to_string()),
            args: func_args_2,
        };
        let stmts = vec![
            Stmt::Expr(func_call_1),
            Stmt::Expr(func_call_2)
        ];
        let inst = Some(Assign {
            typ: Some(DeclType::Let),
            ident: Ident("idx".to_string()),
            expr: Expr::Literal(Literal::Number(3.0))
        });
        let chk = Some(Expr::Comp {
            op: CompOp("<".to_string()),
            l: box Expr::Var("idx".to_string()),
            r: box Expr::Literal(Literal::Number(10.0))
        });
        let incr = Some(Expr::Comp {
            op: CompOp("+=".to_string()),
            l: box Expr::Var("idx".to_string()),
            r: box Expr::Literal(Literal::Number(1.0))
        });
        let for_loop = Stmt::ForLoop {
            inst,
            chk,
            incr,
            stmts,
        };
        assert_eq!(
            for_loop.gen(&ctx),
            "\
for (let idx = 3; (idx) < (10); (idx) += (1)) {
console.log(\"OK\");
alert((3) + (4));
}\
"
        )
    }

    #[test]
    fn named_export() {
        let stmt = Stmt::Expr(Expr::Var("xiaosi".to_string()));
        let export_stmt = Stmt::Export{ is_default: false, stmt: box stmt };
        assert_eq!(export_stmt.gen(&GenContext{}), "export xiaosi;")
    }

    fn default_export() {
        let stmt = Stmt::Expr(Expr::Var("xiaosi".to_string()));
        let export_stmt = Stmt::Export{ is_default: true, stmt: box stmt };
        assert_eq!(export_stmt.gen(&GenContext{}), "export default xiaosi;")
    }

    #[test]
    fn xiaosi_class() {
        let ident = Ident("XiaoSi".to_string());
        let constructor = Constructor {
            params: vec![Ident("url".to_string()), Ident("params".to_string())],
            stmts: vec![Stmt::Expr(
                Expr::FuncCall {
                    func: box Expr::Var("console.log".to_string()),
                    args: vec![Expr::Literal(Literal::String("Hello world!".to_string()))]
                }
            )],
        };
        let methods = vec![Method {
            ident: Ident("loves".to_string()),
            params: vec!["singoi".to_string()],
            stmts: vec![Stmt::Return(Expr::Literal(Literal::Boolean(true)))],
            is_async: true,
        }];
        let getters = vec![Getter {
            ident: Ident("age".to_string()),
            stmts: vec![Stmt::Return(Expr::Literal(Literal::Number(23.0)))]
        }];
        let xiaosi_class = Class {
            ident,
            extends: Some(Ident("Parent".to_string())),
            constructor: Some(constructor),
            methods,
            getters,
        };
        println!("{}", xiaosi_class.gen(&GenContext {}));
        assert_eq!(
            xiaosi_class.gen(&GenContext {}),
            "\
class XiaoSi extends Parent {
constructor(url, params) {
console.log(\"Hello world!\");
}
async loves(singoi) {
return true;
}
get age() {
return 23;
}
}"
        );
    }

    #[test]
    fn xiaosi_import_all() {
        let imp = Import {
            def: Some(Ident("XiaoSi".to_string())),
            imps: Some(Left(ImportStar())),
            path: "xiaosi".to_string(),
        };
        assert_eq!(
            "\
             import XiaoSi, * from \"xiaosi\";",
            imp.gen(&GenContext {})
        );
    }

    #[test]
    fn xiaosi_import_named() {
        let imp = Import {
            def: None,
            imps: Some(Right(vec![
                ImportName::Simple(Ident("alpha".to_string())),
                ImportName::Alias(Ident("beta".to_string()), Ident("bravo".to_string())),
            ])),
            path: "xiaosi".to_string()
        };
        assert_eq!(
            "\
             import {alpha, beta as bravo} from \"xiaosi\";",
            imp.gen(&GenContext {})
        );
    }

    #[test]
    fn xiaosi_import_side_effect_only() {
        let imp = Import {
            def: None,
            imps: None,
            path: "xiaosi".to_string(),
        };
        assert_eq!(
            "\
             import \"xiaosi\";",
            imp.gen(&GenContext {})
        );
    }

    #[test]
    fn xiaosi_code() {
        let stmts = vec![
            Stmt::Import(Import{
                def: Some(Ident("Hello".to_string())),
                imps: Some(Left(ImportStar())),
                path: "hello".to_string()
            }),
            Stmt::Class(Class{
                ident: Ident("XiaoSi".to_string()),
                extends: None,
                constructor: None,
                methods: Vec::new(),
                getters: Vec::new()
            })
        ];
        let code = Code {
            stmts: stmts
        };
        println!("{}", code.gen(&GenContext{}));
    }
}
