#[derive(Debug)]
pub enum Either<L, R> {
    Left(L),
    Right(R)
}

// pub fn indent(s: String, by: String) -> String {
//     s.split("\n").map(|line| by + line).concat("\n")
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indent() {
        assert_eq!(1, 1);
    }
}
