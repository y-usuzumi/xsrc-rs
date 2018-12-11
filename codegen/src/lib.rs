#![feature(box_syntax)]

#[macro_use]
extern crate serde_derive;
pub mod javascript;
pub mod typescript;
pub mod utils;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
