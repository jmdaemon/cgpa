use csv::{Reader, ReaderBuilder, Trim};
use serde::de::Visitor;
use serde::{de, Deserialize, Deserializer};

pub type CSVReader<'a> = Reader<&'a [u8]>;

pub fn create_csv_reader(content: &[u8]) -> CSVReader {
    ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .from_reader(content)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Percent {
    pub percent: String,
    pub value: u8,
}

impl Percent {
    pub fn to_weight(&self) -> f64 {
        self.value as f64 / 100.0
    }
}

pub(crate) struct PercentVisitor;

impl<'de> Visitor<'de> for PercentVisitor {
    type Value = Percent;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a whole number percent between 0-100 (e.g 55%)")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let s = v.split("%").collect::<Vec<&str>>().join("").to_string();
        let value = s.parse::<u8>().expect("Error: Unable to parse percent");
        Ok(Percent {
            percent: v.to_owned(),
            value,
        })
    }
}

impl<'de> Deserialize<'de> for Percent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PercentVisitor)
    }
}
