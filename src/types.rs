use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct SysctlValue {
    pub value: String,
    pub ignore_error: bool,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SchemaType {
    String,
    Boolean,
    Number,
}

impl SchemaType {
    pub fn from_str(value: &str) -> Self {
        if value == "true" || value == "false" {
            return SchemaType::Boolean;
        }
        if value.parse::<f32>().is_ok() {
            return SchemaType::Number;
        }

        SchemaType::String
    }
}

#[test]
fn schema_type_from_str() {
    assert_eq!(SchemaType::from_str("true"), SchemaType::Boolean);
    assert_eq!(SchemaType::from_str("false"), SchemaType::Boolean);
    assert_eq!(SchemaType::from_str("42"), SchemaType::Number);
    assert_eq!(SchemaType::from_str("3.14"), SchemaType::Number);
    assert_eq!(SchemaType::from_str("hello"), SchemaType::String);
}

impl Display for SchemaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaType::String => write!(f, "string"),
            SchemaType::Boolean => write!(f, "bool"),
            SchemaType::Number => write!(f, "number"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SchemaEntry {
    pub name: String,
    pub schema_type: SchemaType,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Schema {
    pub entries: Vec<SchemaEntry>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValidationError {
    MissingKey(String),
    UnknownKey(String),
    WrongType {
        key_name: String,
        expect: SchemaType,
        actual: SchemaType,
    },
    TooLongLine(String),
}
