#![feature(type_ascription)]
pub mod javascript;
pub mod typescript;

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
