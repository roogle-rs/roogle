use log_derive::logfn;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    character::complete::{alpha1, alphanumeric1, multispace0, multispace1},
    combinator::{eof, map, opt, recognize},
    error::{ContextError, ParseError},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded},
    IResult,
};

use crate::types::{Argument, FnDecl, FnRetTy, Function, PrimitiveType, Query, QueryKind, Type};

type Symbol = String;

#[logfn(info, fmt = "Parsing query finished: {:?}")]
pub fn parse_query<'a, E>(i: &'a str) -> IResult<&'a str, Query, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + std::fmt::Debug,
{
    parse_function_query(i)
}

fn parse_symbol<'a, E>(i: &'a str) -> IResult<&'a str, Symbol, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    map(
        recognize(pair(
            alt((tag("_"), alpha1)),
            many0(alt((tag("_"), alphanumeric1))),
        )),
        |symbol: &str| symbol.to_string(),
    )(i)
}

fn parse_function_query<'a, E>(i: &'a str) -> IResult<&'a str, Query, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let (i, _) = tag("fn")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, name) = opt(parse_symbol)(i)?;
    let (i, decl) = opt(parse_function)(i)?;

    let query = Query {
        name,
        kind: decl.map(QueryKind::FunctionQuery),
    };
    Ok((i, query))
}

fn parse_function<'a, E>(i: &'a str) -> IResult<&'a str, Function, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let (i, decl) = parse_function_decl(i)?;

    let function = Function { decl };
    Ok((i, function))
}

fn parse_function_decl<'a, E>(i: &'a str) -> IResult<&'a str, FnDecl, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let (i, inputs) = delimited(
        char('('),
        alt((map(tag(".."), |_| None), opt(parse_arguments))),
        char(')'),
    )(i)?;
    let (i, output) = alt((map(char('_'), |_| None), opt(parse_output)))(i)?;

    let decl = FnDecl { inputs, output };
    Ok((i, decl))
}

fn parse_arguments<'a, E>(i: &'a str) -> IResult<&'a str, Vec<Argument>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    separated_list0(
        char(','),
        preceded(
            multispace0,
            alt((
                parse_argument,
                map(char('_'), |_| Argument {
                    ty: None,
                    name: None,
                }),
            )),
        ),
    )(i)
}

fn parse_argument<'a, E>(i: &'a str) -> IResult<&'a str, Argument, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let (i, name) = alt((map(char('_'), |_| None), opt(parse_symbol)))(i)?;
    let (i, _) = char(':')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, ty) = alt((map(char('_'), |_| None), opt(parse_type)))(i)?;

    let arg = Argument { ty, name };
    Ok((i, arg))
}

fn parse_output<'a, E>(i: &'a str) -> IResult<&'a str, FnRetTy, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    preceded(
        multispace0,
        alt((
            map(preceded(tag("->"), parse_type), FnRetTy::Return),
            map(eof, |_| FnRetTy::DefaultReturn),
        )),
    )(i)
}

fn parse_type<'a, E>(i: &'a str) -> IResult<&'a str, Type, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    preceded(multispace0, map(parse_primitive_type, Type::Primitive))(i)
}

fn parse_primitive_type<'a, E>(i: &'a str) -> IResult<&'a str, PrimitiveType, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    alt((
        map(tag("isize"), |_| PrimitiveType::Isize),
        map(tag("usize"), |_| PrimitiveType::Usize),
    ))(i)
}
