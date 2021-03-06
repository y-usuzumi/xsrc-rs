#![feature(try_blocks)]
#![feature(custom_attribute)]
#![feature(non_ascii_idents)]
#![feature(box_syntax)]

pub mod rewriter;
pub mod schema;
pub mod se_parser;
pub mod transformer;

#[cfg(test)]
mod tests {
    #[test]
    fn test_ownership() {
        #[derive(Debug, PartialEq)]
        struct X {
            x: u32,
        }

        let x1 = X { x: 1 };
        let x2 = X { x: 2 };
        let xs = vec![x1, x2];
        for x in &xs {
            let z = x;
        }
        assert_eq!(xs[0], X { x: 1 })
    }
}
