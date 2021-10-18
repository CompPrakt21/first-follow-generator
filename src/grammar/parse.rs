use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
use nom::combinator::all_consuming;
use nom::multi::fold_many0;
use nom::multi::separated_list1;
use nom::sequence::tuple;
use nom::IResult;
use nom::{character::complete::*, Parser};

use super::{Grammar, Symbol};

pub fn parse_grammar<'a>(src: &'a str) -> Grammar {
    let rules = all_consuming(separated_list1(
        tuple((multispace0, tag(","), multispace0)),
        parse_rule,
    ))
    .parse(src);

    let rules = rules.unwrap().1.into_iter().collect::<HashMap<_, _>>();

    Grammar { rules }
}

fn parse_rule<'a>(src: &'a str) -> IResult<&'a str, (String, Symbol)> {
    tuple((
        parse_non_terminal,
        multispace0,
        tag("->"),
        multispace0,
        parse_alternate,
    ))
    .map(|(left, _, _, _, right)| (left, right))
    .parse(src)
}

fn parse_alternate<'a>(src: &'a str) -> IResult<&'a str, Symbol> {
    separated_list1(tuple((multispace0, tag("|"), multispace0)), parse_sequence)
        .map(|mut alt| match alt.len() {
            0 => Symbol::Epsilon,
            1 => alt.pop().unwrap(),
            _ => Symbol::Alternate(alt),
        })
        .parse(src)
}

fn parse_sequence<'a>(src: &'a str) -> IResult<&'a str, Symbol> {
    separated_list1(multispace1, parse_optional_repeat)
        .map(|mut seq| match seq.len() {
            0 => Symbol::Epsilon,
            1 => seq.pop().unwrap(),
            _ => Symbol::Sequence(seq),
        })
        .parse(src)
}

fn parse_optional_repeat<'a>(src: &'a str) -> IResult<&'a str, Symbol> {
    parse_atom
        .flat_map(|sym| {
            fold_many0(
                alt((char('*'), char('?'))),
                move || sym.clone(),
                |sym, c| match c {
                    '*' => Symbol::Repeat(Box::new(sym)),
                    '?' => Symbol::Optional(Box::new(sym)),
                    _ => unreachable!(),
                },
            )
        })
        .parse(src)
}

fn parse_atom<'a>(src: &'a str) -> IResult<&'a str, Symbol> {
    alt((
        parse_terminal,
        parse_eps,
        parse_non_terminal.map(Symbol::NonTerminal),
        tuple((
            tag("("),
            multispace0,
            parse_alternate,
            multispace0,
            tag(")"),
        ))
        .map(|(_, _, a, _, _)| a),
    ))
    .parse(src)
}

fn parse_non_terminal<'a>(src: &'a str) -> IResult<&'a str, String> {
    alphanumeric1.map(|s: &str| s.into()).parse(src)
}

fn parse_eps<'a>(src: &'a str) -> IResult<&'a str, Symbol> {
    tag("​ɛ").map(|_| Symbol::Epsilon).parse(src)
}

fn parse_terminal<'a>(src: &'a str) -> IResult<&'a str, Symbol> {
    tuple((
        tag("'"),
        take_until("'").map(|s: &str| Symbol::Terminal(s.into())),
        tag("'"),
    ))
    .map(|(_, t, _)| t)
    .parse(src)
}
