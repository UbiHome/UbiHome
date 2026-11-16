//! A minimal, order-preserving YAML document model.
//!
//! The editor needs to round-trip arbitrary UbiHome configurations without
//! knowing every platform's schema, and must keep the key order the user
//! wrote. `serde-saphyr` handles parsing and emitting; this module only
//! provides a self-describing value type that preserves mapping order.

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Yaml {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Seq(Vec<Yaml>),
    Map(Vec<(String, Yaml)>),
}

impl Default for Yaml {
    fn default() -> Self {
        Yaml::Map(Vec::new())
    }
}

impl Yaml {
    pub fn parse(text: &str) -> Result<Yaml, String> {
        if text.trim().is_empty() {
            return Ok(Yaml::Map(Vec::new()));
        }
        serde_saphyr::from_str::<Yaml>(text).map_err(|e| e.to_string())
    }

    /// Emits the document in the style used by the UbiHome docs: indented
    /// list items and `key:` (instead of `key: null`) for empty values.
    pub fn to_text(&self) -> Result<String, String> {
        if matches!(self, Yaml::Map(entries) if entries.is_empty()) {
            return Ok(String::new());
        }
        let options = serde_saphyr::ser_options! { compact_list_indent: false };
        let text =
            serde_saphyr::to_string_with_options(self, options).map_err(|e| e.to_string())?;
        let mut out = String::with_capacity(text.len());
        for line in text.lines() {
            // Strings spelled `null` are always quoted by the emitter, so an
            // unquoted trailing `: null` is a real null value.
            out.push_str(
                line.strip_suffix(" null")
                    .filter(|l| l.ends_with(':'))
                    .unwrap_or(line),
            );
            out.push('\n');
        }
        Ok(out)
    }

    pub fn as_map(&self) -> Option<&Vec<(String, Yaml)>> {
        match self {
            Yaml::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_map_mut(&mut self) -> Option<&mut Vec<(String, Yaml)>> {
        match self {
            Yaml::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_seq(&self) -> Option<&Vec<Yaml>> {
        match self {
            Yaml::Seq(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Yaml::String(s) => Some(s),
            _ => None,
        }
    }

    /// Looks up a key of a mapping.
    pub fn get(&self, key: &str) -> Option<&Yaml> {
        self.as_map()?
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Yaml> {
        self.as_map_mut()?
            .iter_mut()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// Sets a key of a mapping (appending it if missing). No-op on non-maps.
    pub fn set(&mut self, key: &str, value: Yaml) {
        if let Some(map) = self.as_map_mut() {
            if let Some(entry) = map.iter_mut().find(|(k, _)| k == key) {
                entry.1 = value;
            } else {
                map.push((key.to_string(), value));
            }
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<Yaml> {
        let map = self.as_map_mut()?;
        let index = map.iter().position(|(k, _)| k == key)?;
        Some(map.remove(index).1)
    }

    /// Short single-line rendering used for summaries in the UI.
    pub fn summary(&self) -> String {
        match self {
            Yaml::Null => "null".into(),
            Yaml::Bool(b) => b.to_string(),
            Yaml::Int(i) => i.to_string(),
            Yaml::Float(f) => f.to_string(),
            Yaml::String(s) => s.clone(),
            Yaml::Seq(s) => format!("[{} items]", s.len()),
            Yaml::Map(m) => format!("{{{} keys}}", m.len()),
        }
    }

    pub fn kind(&self) -> Kind {
        match self {
            Yaml::Null => Kind::Null,
            Yaml::Bool(_) => Kind::Bool,
            Yaml::Int(_) => Kind::Int,
            Yaml::Float(_) => Kind::Float,
            Yaml::String(_) => Kind::String,
            Yaml::Seq(_) => Kind::Seq,
            Yaml::Map(_) => Kind::Map,
        }
    }

    /// Converts the value to another kind, keeping as much information as
    /// reasonably possible.
    pub fn convert(&self, kind: Kind) -> Yaml {
        if self.kind() == kind {
            return self.clone();
        }
        let text = match self {
            Yaml::Seq(_) | Yaml::Map(_) | Yaml::Null => String::new(),
            other => other.summary(),
        };
        match kind {
            Kind::Null => Yaml::Null,
            Kind::Bool => Yaml::Bool(matches!(text.as_str(), "true" | "1")),
            Kind::Int => Yaml::Int(text.parse::<f64>().map(|f| f as i64).unwrap_or(0)),
            Kind::Float => Yaml::Float(text.parse().unwrap_or(0.0)),
            Kind::String => Yaml::String(text),
            Kind::Seq => Yaml::Seq(Vec::new()),
            Kind::Map => Yaml::Map(Vec::new()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Null,
    Bool,
    Int,
    Float,
    String,
    Seq,
    Map,
}

impl Kind {
    pub const ALL: [Kind; 7] = [
        Kind::String,
        Kind::Int,
        Kind::Float,
        Kind::Bool,
        Kind::Map,
        Kind::Seq,
        Kind::Null,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Kind::Null => "null",
            Kind::Bool => "bool",
            Kind::Int => "int",
            Kind::Float => "float",
            Kind::String => "text",
            Kind::Seq => "list",
            Kind::Map => "map",
        }
    }
}

impl Serialize for Yaml {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Yaml::Null => serializer.serialize_unit(),
            Yaml::Bool(b) => serializer.serialize_bool(*b),
            Yaml::Int(i) => serializer.serialize_i64(*i),
            Yaml::Float(f) => serializer.serialize_f64(*f),
            Yaml::String(s) => serializer.serialize_str(s),
            Yaml::Seq(items) => {
                let mut seq = serializer.serialize_seq(Some(items.len()))?;
                for item in items {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Yaml::Map(entries) => {
                let mut map = serializer.serialize_map(Some(entries.len()))?;
                for (k, v) in entries {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Yaml {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(YamlVisitor)
    }
}

struct YamlVisitor;

impl<'de> Visitor<'de> for YamlVisitor {
    type Value = Yaml;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("any YAML value")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Yaml, E> {
        Ok(Yaml::Bool(v))
    }
    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Yaml, E> {
        Ok(Yaml::Int(v))
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Yaml, E> {
        Ok(i64::try_from(v)
            .map(Yaml::Int)
            .unwrap_or(Yaml::Float(v as f64)))
    }
    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Yaml, E> {
        Ok(Yaml::Float(v))
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Yaml, E> {
        Ok(Yaml::String(v.to_string()))
    }
    fn visit_string<E: de::Error>(self, v: String) -> Result<Yaml, E> {
        Ok(Yaml::String(v))
    }
    fn visit_unit<E: de::Error>(self) -> Result<Yaml, E> {
        Ok(Yaml::Null)
    }
    fn visit_none<E: de::Error>(self) -> Result<Yaml, E> {
        Ok(Yaml::Null)
    }
    fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Yaml, D::Error> {
        Yaml::deserialize(d)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Yaml, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element()? {
            items.push(item);
        }
        Ok(Yaml::Seq(items))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Yaml, A::Error> {
        let mut entries: Vec<(String, Yaml)> = Vec::new();
        while let Some(key) = map.next_key::<Yaml>()? {
            let key = match key {
                Yaml::Null => String::new(),
                other => other.summary(),
            };
            let value: Yaml = map.next_value()?;
            entries.push((key, value));
        }
        Ok(Yaml::Map(entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = r#"
ubihome:
  name: test_device
api:
  port: 8053
mdns:
button:
  - platform: shell
    id: my_button
    name: Write Hello World
    command: "echo 'Hello World!'"
sensor:
  - platform: shell
    name: CPU
    update_interval: 30s
    accuracy_decimals: 2
    filters:
      - round: 1
"#;

    #[test]
    fn round_trip_preserves_order_and_values() {
        let doc = Yaml::parse(CONFIG).unwrap();
        let keys: Vec<_> = doc
            .as_map()
            .unwrap()
            .iter()
            .map(|(k, _)| k.as_str())
            .collect();
        assert_eq!(keys, ["ubihome", "api", "mdns", "button", "sensor"]);
        assert_eq!(doc.get("mdns"), Some(&Yaml::Null));

        let text = doc.to_text().unwrap();
        let reparsed = Yaml::parse(&text).unwrap();
        assert_eq!(doc, reparsed, "emitted:\n{text}");
    }

    #[test]
    fn strings_that_look_like_other_types_stay_strings() {
        let mut doc = Yaml::default();
        doc.set("a", Yaml::String("true".into()));
        doc.set("b", Yaml::String("8053".into()));
        doc.set("c", Yaml::String("".into()));
        let reparsed = Yaml::parse(&doc.to_text().unwrap()).unwrap();
        assert_eq!(doc, reparsed);
    }

    #[test]
    fn emits_docs_style() {
        let doc = Yaml::parse("api:\nswitch:\n  - platform: shell\n    name: null\n").unwrap();
        assert_eq!(
            doc.to_text().unwrap(),
            "api:\nswitch:\n  - platform: shell\n    name:\n"
        );
        let mut doc = Yaml::default();
        doc.set("a", Yaml::String("null".into()));
        assert_eq!(Yaml::parse(&doc.to_text().unwrap()).unwrap(), doc);
    }

    #[test]
    fn empty_document_is_empty_map() {
        assert_eq!(Yaml::parse("  \n").unwrap(), Yaml::Map(vec![]));
    }
}
