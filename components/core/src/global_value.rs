use serde::{Deserialize, Serialize};
use std::fmt;

/// The value of a `globals` variable, or a value being assigned to one via
/// `globals.set`.
///
/// This is `#[serde(untagged)]`, so a YAML scalar deserializes into whichever
/// variant it looks like (`true`/`false` into [`GlobalValue::Bool`], `42` into
/// [`GlobalValue::Int`], `1.5` into [`GlobalValue::Float`], anything else -
/// including an explicitly quoted string - into [`GlobalValue::String`]).
/// Users therefore don't need to quote booleans or numbers in `initial_value`
/// or `globals.set`. The runtime's global store reconciles the declared
/// `type:` of a global with the variant actually given, but does not parse a
/// string into another type - a quoted `"42"` is a type mismatch for an
/// `int`-typed global, not a coercion.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GlobalValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

impl fmt::Display for GlobalValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GlobalValue::Bool(value) => write!(f, "{}", value),
            GlobalValue::Int(value) => write!(f, "{}", value),
            GlobalValue::Float(value) => write!(f, "{}", value),
            GlobalValue::String(value) => write!(f, "{}", value),
        }
    }
}
