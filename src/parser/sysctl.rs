use super::util::{equals, hyphen, skip0, token};
use crate::types::SysctlValue;
use nom::{
    bytes::complete::{take_till, take_while},
    combinator::{eof, map, opt},
    multi::many0,
    sequence::{delimited, terminated, tuple},
    IResult,
};
use std::collections::HashMap;

// = や空白以外の任意の連続した文字
// 例) hoge, console.log /var/log
fn parse_key(input: &str) -> IResult<&str, &str> {
    token(take_while(|c: char| !c.is_whitespace() && c != '='))(input)
}

fn parse_value(input: &str) -> IResult<&str, &str> {
    // 行の終わりまで読み込んでtrimする
    map(
        token(take_till(|c: char| c == '\r' || c == '\n')),
        |s: &str| s.trim(),
    )(input)
}

// key = value の部分
// 例) endpoint = localhost:3000
fn parse_key_value(input: &str) -> IResult<&str, (String, SysctlValue)> {
    map(
        tuple((opt(hyphen), parse_key, equals, parse_value)),
        |(opt_hyphen, k, _, v)| {
            let ignore_error = opt_hyphen.is_some();
            (
                k.to_owned(),
                SysctlValue {
                    value: v.to_string(),
                    ignore_error,
                },
            )
        },
    )(input)
}

pub fn parse_sysctl(input: &str) -> IResult<&str, HashMap<String, SysctlValue>> {
    map(
        terminated(many0(delimited(skip0, parse_key_value, skip0)), eof),
        |kvs| kvs.into_iter().collect::<HashMap<_, _>>(),
    )(input)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_key() {
        assert_eq!(parse_key("key=value"), Ok(("=value", "key")));
        assert_eq!(parse_key("-key=value"), Ok(("=value", "-key")));
        assert_eq!(parse_key(" key=value"), Ok(("=value", "key")));
        assert_eq!(parse_key("\tkey=value"), Ok(("=value", "key")));
        assert_eq!(parse_key("key =value"), Ok((" =value", "key")));
    }

    #[test]
    fn test_value() {
        assert_eq!(parse_value("value\n"), Ok(("\n", "value")));
        assert_eq!(parse_value("value "), Ok(("", "value")));
        assert_eq!(parse_value(" value "), Ok(("", "value")));
        assert_eq!(parse_value(" value\n"), Ok(("\n", "value")));
    }

    #[test]
    fn test_key_value() {
        assert_eq!(
            parse_key_value("-key = value\n"),
            Ok((
                "\n",
                (
                    "key".to_string(),
                    SysctlValue {
                        value: "value".to_string(),
                        ignore_error: true
                    }
                )
            ))
        );
        assert_eq!(
            parse_key_value("key=value\n"),
            Ok((
                "\n",
                (
                    "key".to_string(),
                    SysctlValue {
                        value: "value".to_string(),
                        ignore_error: false
                    }
                )
            ))
        );
        assert_eq!(
            parse_key_value("key = value \n"),
            Ok((
                "\n",
                (
                    "key".to_string(),
                    SysctlValue {
                        value: "value".to_string(),
                        ignore_error: false
                    }
                )
            ))
        );
        assert_eq!(
            parse_key_value("-key=value"),
            Ok((
                "",
                (
                    "key".to_string(),
                    SysctlValue {
                        value: "value".to_string(),
                        ignore_error: true
                    }
                )
            ))
        );
    }

    #[test]
    fn test_parse_sysctl() {
        let input = "
            # comment
            key1 = value1
            -key2 = value2
            key3=value3
            key4 =    value4   
            # another comment
        ";
        let expected_output = vec![
            (
                "key1".to_string(),
                SysctlValue {
                    value: "value1".to_string(),
                    ignore_error: false,
                },
            ),
            (
                "key2".to_string(),
                SysctlValue {
                    value: "value2".to_string(),
                    ignore_error: true,
                },
            ),
            (
                "key3".to_string(),
                SysctlValue {
                    value: "value3".to_string(),
                    ignore_error: false,
                },
            ),
            (
                "key4".to_string(),
                SysctlValue {
                    value: "value4".to_string(),
                    ignore_error: false,
                },
            ),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();
        assert_eq!(parse_sysctl(input), Ok(("", expected_output)));
    }
}
