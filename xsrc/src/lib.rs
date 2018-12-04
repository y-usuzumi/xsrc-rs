#![feature(box_syntax)]

#[macro_use] extern crate maplit;
extern crate serde_yaml;
#[macro_use] extern crate serde_derive;
extern crate codegen;

pub mod schema;
pub mod transformer;
pub mod separser;

#[cfg(test)]
mod tests {
    #[test]
    fn test_ownership() {
        #[derive(Debug)]
        struct X { n: i32 }
        let x = X{n: 3};
        let xs = vec![
            Box::new(x),
            Box::new(X{n: 4}),
        ];
        for x in xs.iter() {
            println!("{:?}", *x);
        }
        // println!("{:?}", x);
    }
}
