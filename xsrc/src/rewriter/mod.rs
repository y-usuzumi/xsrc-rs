use super::transformer::ContextBoundedRoot;

pub mod javascript;

pub trait CodeGen {
    fn gen(&self, root: ContextBoundedRoot);
}
