use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, Serializer};
use linked_hash_map::LinkedHashMap;
use std::convert::From;
use std::fmt;
use std::fs::File;
use std::ops::Deref;
use std::path::Path;

#[derive(Debug)]
pub enum ParserError {
    IOError(std::io::Error),
    SerdeError(serde_yaml::Error),
}

#[derive(Debug)]
pub struct APIDataMap(pub LinkedHashMap<String, APIData>);

impl Deref for APIDataMap {
    type Target = LinkedHashMap<String, APIData>;

    fn deref(&self) -> &LinkedHashMap<String, APIData> {
        return &self.0;
    }
}

impl From<std::io::Error> for ParserError {
    fn from(e: std::io::Error) -> Self {
        ParserError::IOError(e)
    }
}

impl From<serde_yaml::Error> for ParserError {
    fn from(e: serde_yaml::Error) -> Self {
        ParserError::SerdeError(e)
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
        let mut map = LinkedHashMap::with_capacity(access.size_hint().unwrap_or(0));

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
                }
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
        deserializer.deserialize_map(APIDataMapVisitor {})
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RootSchema {
    #[serde(rename = "$url")]
    pub url: Option<String>,

    #[serde(rename = "$as", default = "RootSchema::default_klsname")]
    pub klsname: String,

    #[serde(flatten)]
    pub apisets: APIDataMap,
}

impl RootSchema {
    fn default_klsname() -> String {
        "XSClient".to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum APIData {
    API(APISchema),
    APISet(APISetSchema),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct APISchema {
    #[serde(rename = "$method", default = "APISchema::default_method")]
    pub method: String,

    #[serde(rename = "$params", default = "APISchema::default_params")]
    pub params: LinkedHashMap<String, Option<String>>,

    #[serde(rename = "$data", default = "APISchema::default_data")]
    pub data: LinkedHashMap<String, Option<String>>,

    #[serde(rename = "$url", default = "APISchema::default_url")]
    pub url: String,
}

impl APISchema {
    fn default_method() -> String {
        "GET".to_string()
    }

    fn default_params() -> LinkedHashMap<String, Option<String>> {
        LinkedHashMap::new()
    }

    fn default_data() -> LinkedHashMap<String, Option<String>> {
        LinkedHashMap::new()
    }

    fn default_url() -> String {
        "${!super.url}".to_string()
    }
}

pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<RootSchema, ParserError> {
    let f = File::open(path)?;
    let result = serde_yaml::from_reader(f)?;
    Ok(result)
}

pub fn parse_str(s: &str) -> Result<RootSchema, ParserError> {
    let result = serde_yaml::from_str(s)?;
    Ok(result)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct APISetSchema {
    #[serde(rename = "$url", default = "APISetSchema::default_url")]
    pub url: String,
    #[serde(flatten)]
    pub apisets: APIDataMap,
}

impl APISetSchema {
    fn default_url() -> String {
        "${!super.url}".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_struct_works() {
        let sample_string = include_str!("../tests/fixtures/sample.yaml");
        let result: Result<RootSchema, _> = serde_yaml::from_str(&sample_string);
        assert!(result.is_ok());
    }

    #[test]
    fn schema_no_root_url_works() {
        let sample_string = include_str!("../tests/fixtures/sample_no_klsname_no_url.yaml");
        let result: RootSchema = serde_yaml::from_str(&sample_string).unwrap();
        assert_eq!(result.klsname, "XSClient".to_string());
    }
}
