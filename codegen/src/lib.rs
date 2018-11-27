#![feature(box_syntax)]

pub mod javascript;
pub mod typescript;
pub mod utils;

pub fn test() {
    println!("CodeGen");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
