use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Value {
    Number(f64),
    Text(String),
    Bool(bool),
    List(Vec<Value>),
    Timestamp(f64),
    Duration(f64),
    /// v1.8: A secret value that is redacted in display output.
    /// Can only be passed to `write` actions and external adapters.
    Secret(String),
    /// v1.8: Represents a field whose value is unknown (e.g. external entity
    /// has lost connectivity or sync timeout has expired).
    Unknown,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "Number",
            Value::Text(_) => "Text",
            Value::Bool(_) => "Boolean",
            Value::List(_) => "List",
            Value::Timestamp(_) => "Timestamp",
            Value::Duration(_) => "Duration",
            Value::Secret(_) => "Secret",
            Value::Unknown => "Unknown",
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let Value::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        if let Value::Text(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<&Vec<Value>> {
        if let Value::List(l) = self {
            Some(l)
        } else {
            None
        }
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Value::Unknown)
    }

    pub fn is_same_type(&self, other: &Value) -> bool {
        matches!(
            (self, other),
            (Value::Number(_), Value::Number(_))
                | (Value::Text(_), Value::Text(_))
                | (Value::Bool(_), Value::Bool(_))
                | (Value::List(_), Value::List(_))
                | (Value::Timestamp(_), Value::Timestamp(_))
                | (Value::Duration(_), Value::Duration(_))
                | (Value::Secret(_), Value::Secret(_))
                | (Value::Unknown, Value::Unknown)
        )
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            Value::Text(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Timestamp(t) => write!(f, "Timestamp({})", t),
            Value::Duration(d) => {
                let d = *d;
                if d < 60.0 {
                    write!(f, "{}s", d)
                } else if d < 3600.0 {
                    let m = (d / 60.0).floor();
                    let s = d % 60.0;
                    if s > 0.0 {
                        write!(f, "{}m {}s", m, s)
                    } else {
                        write!(f, "{}m", m)
                    }
                } else if d < 86400.0 {
                    let h = (d / 3600.0).floor();
                    let m = ((d % 3600.0) / 60.0).floor();
                    if m > 0.0 {
                        write!(f, "{}h {}m", h, m)
                    } else {
                        write!(f, "{}h", h)
                    }
                } else {
                    let days = (d / 86400.0).floor();
                    let h = ((d % 86400.0) / 3600.0).floor();
                    if h > 0.0 {
                        write!(f, "{}d {}h", days, h)
                    } else {
                        write!(f, "{}d", days)
                    }
                }
            }
            Value::Secret(_) => write!(f, "***SECRET***"),
            Value::Unknown => write!(f, "unknown"),
        }
    }
}
