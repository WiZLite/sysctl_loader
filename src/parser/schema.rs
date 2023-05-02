use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    combinator::{eof, map},
    multi::many0,
    sequence::{preceded, separated_pair, terminated},
    IResult,
};

use crate::types::{Schema, SchemaEntry, SchemaType};

use super::util::{colon, skip0, token};

// : や空白以外の任意の連続した文字
// 例) hoge, console.log /var/log
fn schema_key(input: &str) -> IResult<&str, &str> {
    token(take_while(|c: char| !c.is_whitespace() && c != ':'))(input)
}

// スキーマの型部分をパーサー
// 空白なども飛ばさず、純粋に文字が
fn schema_type(input: &str) -> IResult<&str, SchemaType> {
    token(alt((
        map(token(tag("string")), |_| SchemaType::String),
        map(token(tag("bool")), |_| SchemaType::Boolean),
        map(token(tag("number")), |_| SchemaType::Number),
    )))(input)
}

// key: type の部分
// 例) endpoint: string
fn schema_entry(input: &str) -> IResult<&str, SchemaEntry> {
    map(
        separated_pair(schema_key, colon, schema_type),
        |(key, schema_type)| SchemaEntry {
            name: key.to_owned(),
            schema_type,
        },
    )(input)
}

pub fn parse_schema(input: &str) -> IResult<&str, Schema> {
    map(
        terminated(many0(schema_entry), preceded(skip0, eof)),
        |entries| Schema { entries },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_key() {
        assert_eq!(schema_key("key: value"), Ok((": value", "key")));
    }

    #[test]
    fn test_schema_type() {
        assert_eq!(schema_type("string"), Ok(("", SchemaType::String)));
        assert_eq!(schema_type("bool"), Ok(("", SchemaType::Boolean)));
        assert_eq!(schema_type("number"), Ok(("", SchemaType::Number)));
        assert!(schema_type("invalid").is_err(),);
    }

    #[test]
    fn test_schema_entry() {
        assert_eq!(
            schema_entry("key: string"),
            Ok((
                "",
                SchemaEntry {
                    name: "key".to_owned(),
                    schema_type: SchemaType::String
                }
            ))
        );
        assert_eq!(
            schema_entry("key : bool"),
            Ok((
                "",
                SchemaEntry {
                    name: "key".to_owned(),
                    schema_type: SchemaType::Boolean
                }
            ))
        );
        assert_eq!(
            schema_entry("key  :number"),
            Ok((
                "",
                SchemaEntry {
                    name: "key".to_owned(),
                    schema_type: SchemaType::Number
                }
            ))
        );
        assert!(schema_entry("key: invalid").is_err());
    }

    #[test]
    fn test_parse_schema() {
        assert_eq!(
            parse_schema("key1: string key2: number key3: bool"),
            Ok((
                "",
                Schema {
                    entries: vec![
                        SchemaEntry {
                            name: "key1".to_owned(),
                            schema_type: SchemaType::String
                        },
                        SchemaEntry {
                            name: "key2".to_owned(),
                            schema_type: SchemaType::Number
                        },
                        SchemaEntry {
                            name: "key3".to_owned(),
                            schema_type: SchemaType::Boolean
                        },
                    ]
                }
            ))
        );
        assert_eq!(parse_schema(""), Ok(("", Schema { entries: vec![] })));
    }
}
