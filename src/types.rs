//! Represent a Edge Router VyOS/Vyatta file

/// Represent a value in a Edge Router VyOS/Vyatta file
#[derive(Debug, Eq, PartialEq)]
pub enum Value<'a> {
    /// A object with keys and values
    Object(Vec<ObjectValue<'a>>),

    /// A quoted string
    String(&'a str),

    /// An unquoted string
    UnquotedString(&'a str),

    /// A boolean value
    Boolean(bool),

    /// A null value
    Null,
}

/// Represent an object value in a Edge Router VyOS/Vyatta file
#[derive(Debug, Eq, PartialEq)]
pub enum ObjectValue<'a> {
    /// A key without a value
    Key(&'a str),

    /// A key with a value
    KeyValue(&'a str, Value<'a>),

    /// A key with a value that is an object
    ObjectKeyValue(&'a str, &'a str, Value<'a>),
}

/// Represent a Edge Router VyOS/Vyatta file
#[derive(Debug, Eq, PartialEq)]
pub struct File<'a> {
    /// The values in the file
    pub values: Value<'a>,

    /// The trailing lines in the file
    pub trailing_lines: Vec<&'a str>,
}

impl File<'_> {
    #[must_use]
    /// Serialize the file back to a string
    ///
    /// # Panics
    ///
    /// Panics if the top-level value is not an object.
    pub fn serialize(&self) -> String {
        let values = match &self.values {
            Value::Object(o) => {
                let contents: Vec<_> = o.iter().map(|v| ObjectValue::serialize(v, "")).collect();
                contents.join("")
            }
            // FIXME: Should we actually support cases where the top-level value is not an object?
            _ => panic!("Top level value is not an object"),
        };
        let mut list = vec![values];
        list.push(String::new());
        list.extend(
            self.trailing_lines
                .iter()
                .map(std::string::ToString::to_string),
        );
        list.join("\n")
    }
}

impl Value<'_> {
    #[must_use]
    /// Serialize the value back to a string
    pub fn serialize(&self, indent: &str) -> String {
        match self {
            Self::Object(o) => {
                let new_indent = indent.to_string() + "    ";
                let contents: Vec<_> = o
                    .iter()
                    .map(|v| ObjectValue::serialize(v, &new_indent))
                    .collect();
                format!("{{\n{}{indent}}}", contents.join(""))
            }

            Self::String(s) => format!("\"{s}\""),
            Self::UnquotedString(s) => (*s).to_string(),
            Self::Boolean(b) => format!("{b}"),
            Self::Null => "null".to_string(),
        }
    }
}

impl ObjectValue<'_> {
    #[must_use]
    /// Serialize the object value back to a string
    pub fn serialize(&self, indent: &str) -> String {
        match self {
            Self::Key(name) => format!("{indent}{name}\n"),
            Self::KeyValue(name, value) => {
                format!("{indent}{} {}\n", name, value.serialize(indent))
            }
            Self::ObjectKeyValue(object, name, value) => {
                format!("{indent}{} {} {}\n", object, name, value.serialize(indent))
            }
        }
    }
}
