#[derive(Debug, Eq, PartialEq)]
pub enum Value<'a> {
    Object(Vec<ObjectValue<'a>>),
    String(&'a str),
    UnquotedString(&'a str),
    Boolean(bool),
    Null,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ObjectValue<'a> {
    Key(&'a str),
    KeyValue(&'a str, Value<'a>),
    ObjectKeyValue(&'a str, &'a str, Value<'a>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct File<'a> {
    pub values: Value<'a>,
    pub trailing_lines: Vec<&'a str>,
}

impl File<'_> {
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
        list.push("".to_string());
        list.extend(self.trailing_lines.iter().map(|s| s.to_string()));
        list.join("\n")
    }
}

impl Value<'_> {
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

            Self::String(s) => format!("\"{}\"", s),
            Self::UnquotedString(s) => s.to_string(),
            Self::Boolean(b) => format!("{}", b),
            Self::Null => "null".to_string(),
        }
    }
}

impl ObjectValue<'_> {
    pub fn serialize(&self, indent: &str) -> String {
        match self {
            Self::Key(name) => format!("{indent}{}\n", name),
            Self::KeyValue(name, value) => {
                format!("{indent}{} {}\n", name, value.serialize(indent))
            }
            Self::ObjectKeyValue(object, name, value) => {
                format!("{indent}{} {} {}\n", object, name, value.serialize(indent))
            }
        }
    }
}
