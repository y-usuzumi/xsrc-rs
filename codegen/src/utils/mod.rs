pub use self::Either::{Left, Right};

#[derive(Debug)]
pub enum Either<L, R> {
    Left(L),
    Right(R)
}

impl<L, R> Either<L, R> {
    pub fn is_left(&self) -> bool {
        match self {
            Left(_) => true,
            _ => false
        }
    }

    pub fn is_right(&self) -> bool {
        match self {
            Right(_) => true,
            _ => false
        }
    }
}

pub fn indent(s: &str, by: &str) -> String {
    s.split("\n").map(|line| format!("{}{}", by, line)).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indent_should_work() {
        let code = "\
hello
world";
        let expected = "    hello
    world";
        assert_eq!(indent(code, "    "), expected);
    }
}
