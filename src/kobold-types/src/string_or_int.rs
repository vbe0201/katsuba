use serde::Deserialize;
use smartstring::alias::String;

/// A value that is either a string or an integer.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum StringOrInt {
    /// A string value.
    String(String),
    /// An integer value.
    Int(i64),
}

impl StringOrInt {
    /// Tries to convert this value into an integer, if possible.
    ///
    /// [`StringOrInt::String`] will be parsed into an integer value,
    /// whereas [`StringOrInt::Int`] will be returned as-is.
    ///
    /// On success, the resulting value will be returned, [`None`]
    /// otherwise.
    pub fn to_int(&self) -> Option<i64> {
        match self {
            &StringOrInt::Int(v) => Some(v),
            StringOrInt::String(s) => s.parse().ok(),
        }
    }

    /// Compares a given `rhs` integer to the `self` value.
    ///
    /// If self is a string, `rhs` is expected to be the value
    /// matching the numeric string representation in self.
    pub fn compare_to_int(&self, rhs: i64) -> bool {
        match self {
            &StringOrInt::Int(v) => v == rhs,
            StringOrInt::String(s) => s.parse().map(|v: i64| v == rhs).unwrap_or(false),
        }
    }

    /// Compares a given `rhs` string to the `self` value.
    ///
    /// If self is an integer, `rhs` is expected to be the
    /// string representation of the same value.
    pub fn compare_to_string(&self, rhs: &str) -> bool {
        match self {
            StringOrInt::String(s) => s == rhs,
            &StringOrInt::Int(v) => rhs.parse().map(|rhs: i64| v == rhs).unwrap_or(false),
        }
    }
}
