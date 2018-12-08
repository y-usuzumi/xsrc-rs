#![feature(box_syntax)]

#[macro_use] extern crate maplit;
extern crate serde_yaml;
#[macro_use] extern crate serde_derive;
extern crate codegen;

pub mod schema;
pub mod se_parser;
pub mod transformer;
