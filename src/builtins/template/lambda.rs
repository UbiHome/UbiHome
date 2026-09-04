use garde::Validate;
use serde::Deserialize;

/// A YAML-expressed state source for a template entity's `lambda`. This
/// avoids C++ lambdas: the entity's state is read directly from a global.
///
/// Shared by every builtin template entity that supports a `lambda`
/// (currently the template switch and template number).
#[derive(Clone, Debug, Deserialize, Validate)]
pub enum LambdaExpr {
    /// Report the entity state from a [`globals`](crate::builtins::globals)
    /// variable.
    #[serde(rename = "globals.get")]
    GlobalsGet(#[garde(ascii)] String),
}

impl LambdaExpr {
    /// The id of the global this lambda reads from.
    pub fn global_id(&self) -> &str {
        match self {
            LambdaExpr::GlobalsGet(id) => id.as_str(),
        }
    }
}
