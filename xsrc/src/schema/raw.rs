use std::collections::HashMap;
pub use self::RawSchemaValue::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum RawSchemaValue {
    StringValue(String),
    IntegerValue(i64),
    FloatValue(f64),
    ChildSchema(RawSchema),
}

type RawSchema = HashMap<String, RawSchemaValue>;


#[cfg(test)]
mod tests {
    use std::io;
    use std::io::prelude::*;
    use std::env;
    use std::fs::File;
    use super::*;

    fn load_sample() -> io::Result<String> {
        let mut f = File::open("tests/fixtures/sample.yaml")?;
        let mut content = String::new();
        f.read_to_string(&mut content)?;
        Ok(content)
    }

    #[test]
    fn schema_struct_works() {
        println!("{:?}", env::current_dir());
        let sample_string = load_sample().unwrap();
        let result: Result<RawSchema, _> = serde_yaml::from_str(&sample_string);
        println!("{:?}", result);
    }
}