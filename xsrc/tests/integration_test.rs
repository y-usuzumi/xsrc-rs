// Note: Per doc, each file in tests directory is a separate crate.
extern crate xsrc;

use std::default::Default;

#[test]
fn test_noname() {
    let schema_str = include_str!("fixtures/sample_no_klsname_no_url.yaml");
    println!("===== Schema str =====");
    println!("{}", schema_str);
    let root_schema = xsrc::schema::parse_str(schema_str).unwrap();
    println!("===== Schema structure =====");
    println!("{:?}", root_schema);
    let root = xsrc::transformer::transform(root_schema).unwrap();
    println!("===== Context-bounded root =====");
    println!("{:?}", root);
    let gen_ctx = Default::default();
    let code = xsrc::rewriter::javascript::gen(&root, &gen_ctx);
    println!("===== JavaScript Code =====");
    println!("{}", code);
}
