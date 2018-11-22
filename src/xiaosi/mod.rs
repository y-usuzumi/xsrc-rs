use serde_derive::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Xiaosi {
    name: String,
    age: i32,
}

pub fn test() {
    let xiaosi = Xiaosi { name: String::from("小四"), age: 23 };
    let s = serde_yaml::to_string(&xiaosi).unwrap();
    println!("{}", s);
}