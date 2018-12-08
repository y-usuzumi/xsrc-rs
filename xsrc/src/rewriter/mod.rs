use super::parser::AST;

pub trait CodeGen {
    fn gen(&self, ast: AST) -> String;
}

struct JavaScriptCodeGen {}

impl JavaScriptCodeGen {
    fn new() -> Self {
        JavaScriptCodeGen{}
    }
}

impl CodeGen for JavaScriptCodeGen {}
