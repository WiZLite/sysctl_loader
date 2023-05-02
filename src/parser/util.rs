use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{line_ending, multispace1},
    combinator::{eof, map},
    multi::many0,
    sequence::tuple,
    IResult, Parser,
};

// コメントをスキップして残りを返すパーサー
pub fn comment<'a>(s: &str) -> IResult<&str, ()> {
    map(
        tuple((
            alt((tag(";"), tag("#"))),
            take_till(|c: char| c == '\r' || c == '\n'),
            alt((line_ending::<&str, _>, eof)),
        )),
        |(_, _, _)| (),
    )(s)
}

// コメントや空白,改行0文字以上をスキップして、残りを返すパーサー
pub fn skip0(input: &str) -> IResult<&str, ()> {
    map(many0(alt((comment, map(multispace1, |_| ())))), |_| ())(input)
}

// パーサーを受け取って、前の空白を読み飛ばす機能をもったパーサーを返すパーサー
// 空白を気にせず、文法に集中したパーサーを書けるようにするために存在している。
pub fn token<'a, O, F>(
    mut first: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, nom::error::Error<&'a str>>
where
    F: Parser<&'a str, O, nom::error::Error<&'a str>>,
{
    move |input: &'a str| {
        let (s, _) = skip0(input)?;
        first.parse(s)
    }
}

pub fn hyphen(input: &str) -> IResult<&str, &str> {
    token(tag("-"))(input)
}

pub fn equals(input: &str) -> IResult<&str, &str> {
    token(tag("="))(input)
}

pub fn colon(input: &str) -> IResult<&str, &str> {
    token(tag(":"))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_comment() {
        assert_eq!(comment("# this is a comment\n"), Ok(("", ())));
        assert_eq!(comment("; this is a comment"), Ok(("", ())));
    }

    #[test]
    fn test_skip0() {
        assert_eq!(skip0("   \n# a comment\n   \n"), Ok(("", ())));
        assert_eq!(skip0("   # a comment"), Ok(("", ())));
        assert_eq!(skip0("   "), Ok(("", ())));
        assert_eq!(skip0("\n"), Ok(("", ())));
    }
}
