use log_derive::logfn;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::char,
    character::complete::{alpha1, alphanumeric1, multispace0, multispace1},
    combinator::{eof, fail, map, not, opt, recognize, value},
    error::{ContextError, ParseError},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded},
    IResult,
};

use crate::types::*;

type Symbol = String;

#[logfn(info, fmt = "Parsing query finished: {:?}")]
pub fn parse_query<'a>(i: &'a str) -> IResult<&'a str, Query> {
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
        alt((
            value(None, tag("..")),
            opt(parse_arguments),
            value(Some(Vec::new()), not(eof)),
        )),
        char(')'),
    )(i)?;
    let (i, output) = opt(parse_output)(i)?;

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
                value(
                    Argument {
                        ty: None,
                        name: None,
                    },
                    char('_'),
                ),
                map(parse_type, |ty| Argument {
                    ty: Some(ty),
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
    let (i, name) = alt((value(None, char('_')), opt(parse_symbol)))(i)?;
    let (i, _) = char(':')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, ty) = alt((value(None, char('_')), opt(parse_type)))(i)?;

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
            value(FnRetTy::DefaultReturn, eof),
        )),
    )(i)
}

fn parse_type<'a, E>(i: &'a str) -> IResult<&'a str, Type, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    preceded(
        multispace0,
        alt((
            map(parse_primitive_type, Type::Primitive),
            parse_generic_type,
            parse_unresolved_path,
            parse_tuple,
            parse_slice,
            value(Type::Never, char('!')),
            parse_raw_pointer,
            parse_borrowed_ref,
        )),
    )(i)
}

fn parse_tuple<'a, E>(i: &'a str) -> IResult<&'a str, Type, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    map(
        delimited(
            char('('),
            separated_list0(
                char(','),
                preceded(
                    multispace0,
                    alt((value(None, tag("_")), map(parse_type, Some))),
                ),
            ),
            char(')'),
        ),
        Type::Tuple,
    )(i)
}

fn parse_slice<'a, E>(i: &'a str) -> IResult<&'a str, Type, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    map(
        delimited(
            char('['),
            alt((value(None, tag("_")), map(parse_type, Some))),
            char(']'),
        ),
        |ty| Type::Slice(ty.map(Box::new)),
    )(i)
}

fn parse_raw_pointer<'a, E>(i: &'a str) -> IResult<&'a str, Type, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let (i, mutable) = alt((value(true, tag("*mut")), value(false, tag("*const"))))(i)?;
    let (i, type_) = parse_type(i)?;

    Ok((
        i,
        Type::RawPointer {
            mutable,
            type_: Box::new(type_),
        },
    ))
}

fn parse_borrowed_ref<'a, E>(i: &'a str) -> IResult<&'a str, Type, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let (i, mutable) = alt((value(true, tag("&mut")), value(false, tag("&"))))(i)?;
    let (i, type_) = parse_type(i)?;

    Ok((
        i,
        Type::BorrowedRef {
            mutable,
            type_: Box::new(type_),
        },
    ))
}

fn parse_unresolved_path<'a, E>(i: &'a str) -> IResult<&'a str, Type, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let (i, name) = parse_symbol(i)?;
    let (i, args) = opt(parse_generic_args)(i)?;

    Ok((
        i,
        Type::UnresolvedPath {
            name,
            args: args.map(Box::new),
        },
    ))
}

fn parse_generic_args<'a, E>(i: &'a str) -> IResult<&'a str, GenericArgs, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    map(
        delimited(
            char('<'),
            separated_list0(
                char(','),
                preceded(
                    multispace0,
                    alt((
                        value(None, tag("_")),
                        opt(map(parse_type, GenericArg::Type)),
                    )),
                ),
            ),
            char('>'),
        ),
        |args| GenericArgs::AngleBracketed { args },
    )(i)
}

fn parse_generic_type<'a, E>(i: &'a str) -> IResult<&'a str, Type, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let (i, gen) = map(take_while1(|c: char| c.is_ascii_uppercase()), |s: &str| {
        Type::Generic(s.to_owned())
    })(i)?;

    if i.chars().next().map_or(false, |c| c.is_ascii_lowercase()) {
        fail(i)
    } else {
        Ok((i, gen))
    }
}

fn parse_primitive_type<'a, E>(i: &'a str) -> IResult<&'a str, PrimitiveType, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    use PrimitiveType::*;
    alt((
        value(Isize, tag("isize")),
        value(I8, tag("i8")),
        value(I16, tag("i16")),
        value(I32, tag("i32")),
        value(I64, tag("i64")),
        value(I128, tag("i128")),
        value(Usize, tag("usize")),
        value(U8, tag("u8")),
        value(U16, tag("u16")),
        value(U32, tag("u32")),
        value(U64, tag("u64")),
        value(U128, tag("u128")),
        value(F32, tag("f32")),
        value(F64, tag("f64")),
        value(Char, tag("char")),
        value(Bool, tag("bool")),
        value(Str, tag("str")),
    ))(i)
}
