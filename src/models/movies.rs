use serde::{Deserialize, Serialize, Serializer, Deserializer};

#[derive(Debug, Deserialize, Serialize)]
pub struct Category {
    pub category_id: String,
    pub category_name: String,
    pub parent_id: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Movie {
    pub num: u32,
    pub name: String,
    pub stream_type: String,
    pub stream_id: u32,
    pub stream_icon: String,
    // pub rating: StringOrFloat,
    pub rating_5based: f32,
    pub added: Option<String>,
    pub is_adult: String,
    pub category_id: String,
    pub container_extension: String,
    pub custom_sid: String,
    pub direct_source: String,
}

#[derive(Debug)]
pub enum StringOrFloat {
    String(String),
    Float(f64),
}

impl Serialize for StringOrFloat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            StringOrFloat::String(s) => serializer.serialize_str(s),
            StringOrFloat::Float(f) => serializer.serialize_f64(*f),
        }
    }
}

impl<'de> Deserialize<'de> for StringOrFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
        if let Some(s) = value.as_str() {
            Ok(StringOrFloat::String(s.to_string()))
        } else if let Some(f) = value.as_f64() {
            Ok(StringOrFloat::Float(f))
        } else {
            Err(serde::de::Error::custom("Expected string or float"))
        }
    }
}