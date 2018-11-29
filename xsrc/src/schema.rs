use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use serde::ser::{Serialize, Serializer, SerializeMap};
use serde::de::{Deserialize, Deserializer, Visitor, MapAccess};

#[derive(Debug)]
pub struct APIDataMap(HashMap<String, APIData>);

impl Deref for APIDataMap {
    type Target = HashMap<String, APIData>;

    fn deref(&self) -> &HashMap<String, APIData> {
        return &self.0;
    }
}

struct APIDataMapVisitor {}

impl<'de> Visitor<'de> for APIDataMapVisitor {
    type Value = APIDataMap;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("keyed API's or APISet's")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));

        // While there are entries remaining in the input, add them
        // into our map.
        while let Some(key) = access.next_key::<String>()? {
            if key.starts_with("~") {
                // APISet
                let name = String::from(&key[1..]);
                let value = access.next_value::<APISetSchema>()?;
                map.insert(name, APIData::APISet(value));
            } else {
                // API
                let value = access.next_value::<APISchema>()?;
                map.insert(key, APIData::API(value));
            }
        }
        Ok(APIDataMap(map))
    }
}

impl Serialize for APIDataMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in (*self).iter() {
            match v {
                APIData::API(_) => {
                    s.serialize_entry(k, v)?;
                },
                APIData::APISet(_) => {
                    s.serialize_entry(&format!("~{}", k), v)?;
                }
            };
        }
        s.end()
    }
}

impl<'de> Deserialize<'de> for APIDataMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of MyMap.
        deserializer.deserialize_map(APIDataMapVisitor{})
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RootSchema {
    #[serde(rename="$url")]
    url: Option<String>,
    
    #[serde(rename="$as", default="RootSchema::default_klsname")]
    klsname: String,

    #[serde(flatten)]
    apisets: APIDataMap
}

impl RootSchema {
    fn default_klsname() -> String {
        "XSClient".to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum APIData {
    API(APISchema),
    APISet(APISetSchema)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct APISchema {
    #[serde(rename="$method", default="APISchema::default_method")]
    method: String
}

impl APISchema {
    fn default_method() -> String {
        "GET".to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct APISetSchema {
    #[serde(rename="$url")]
    url: Option<String>,
    #[serde(flatten)]
    apisets: APIDataMap
}

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
        let result: Result<RootSchema, _> = serde_yaml::from_str(&sample_string);
        assert!(result.is_ok());
    }
}