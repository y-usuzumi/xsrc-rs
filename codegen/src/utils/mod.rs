#[derive(Debug)]
pub enum Either<L, R> {
    Left(L),
    Right(R)
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
