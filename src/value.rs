use serde::{Deserialize, Serialize};

/// The kind of value a flag or argument accepts.
///
/// `Enum` drives a constrained select-list prompt in the interactive CLI
/// instead of a free-text input — use it whenever the legal values are a
/// known, closed set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "values")]
pub enum ValueKind {
    /// Arbitrary UTF-8 string.
    String,
    /// Signed integer.
    Integer,
    /// Floating-point number.
    Float,
    /// true / false.
    Bool,
    /// Filesystem path — may receive special validation or completion later.
    Path,
    /// Constrained to one of the listed strings.  Drives a select prompt.
    Enum(Vec<String>),
}

/// Describes the value that a [`CommandOption`](crate::types::CommandOption) accepts.
///
/// A `None` value on the parent option means the option is a pure flag
/// (e.g. `--verbose`) and carries no value at all.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionValue {
    /// What type the value must be.
    pub kind: ValueKind,

    /// Whether the value is mandatory.  When `false` the option may be
    /// passed without a value and the default applies.
    pub required: bool,

    /// Short display hint shown in prompts, e.g. `<file>`, `<n>`, `<url>`.
    pub hint: String,

    /// Default used when the option is present but no value is supplied.
    pub default: Option<String>,
}
