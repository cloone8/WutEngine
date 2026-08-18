use core::range::Range;
use core::range::RangeInclusive;
use std::str::FromStr;
use std::sync::LazyLock;

use pest::Parser;
use pest::iterators::Pair;
use pest::pratt_parser::Assoc;
use pest::pratt_parser::Op;
use pest::pratt_parser::PrattParser;
use pest_derive::Parser;

/// Start token for a preprocessor directive. Lines that start with this character will be parsed as a preprocessor directive
const DIRECTIVE_START: &str = "#";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Directive<'a> {
    Name(&'a str),
    KeywordDecl {
        ident: &'a str,
        range: Option<KeywordRange>,
    },
    Import(&'a str),
    If(Expr<'a>),
    Else,
    Endif,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KeywordRange {
    Inclusive(RangeInclusive<u64>),
    Exclusive(Range<u64>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Expr<'a> {
    Ident(&'a str),
    Value(u64),
    Unop {
        op: UnopType,
        expr: Box<Expr<'a>>,
    },
    Binop {
        left: Box<Expr<'a>>,
        op: BinopType,
        right: Box<Expr<'a>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum UnopType {
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BinopType {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    And,
    Or,
}

/// [pest]-based parser for WutEngine shader directives
#[derive(Parser)]
#[grammar = "preprocessor/grammar.pest"]
struct DirectiveParser;

pub(super) fn parse_directive(input: &str) -> Result<Directive<'_>, pest::error::Error<Rule>> {
    let input = input.trim();
    let start_index = usize::from(input.starts_with(DIRECTIVE_START));

    let mut pairs = DirectiveParser::parse(Rule::directive, &input[start_index..])?;

    debug_assert_eq!(2, pairs.len(), "Too many output rules?");

    let pair = pairs.next().unwrap();

    Ok(match pair.as_rule() {
        Rule::directive_name => Directive::Name(pair.as_str()),
        Rule::directive_if => Directive::If(parse_expr(pair.into_inner())),
        Rule::directive_else => Directive::Else,
        Rule::directive_endif => Directive::Endif,
        Rule::directive_import => {
            let inner = pair.into_inner();

            Directive::Import(inner.find_first_tagged("content").unwrap().as_str())
        }
        Rule::directive_decl => {
            let mut inner = pair.into_inner();
            let ident = inner.next().unwrap().as_str();
            let range = inner.next().map(parse_range);

            Directive::KeywordDecl { ident, range }
        }
        other => unreachable!("{:?}", other),
    })
}

fn parse_expr<'a>(pairs: impl Iterator<Item = Pair<'a, Rule>>) -> Expr<'a> {
    static PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
        PrattParser::new()
            .op(Op::infix(Rule::or, Assoc::Left))
            .op(Op::infix(Rule::and, Assoc::Left))
            .op(Op::infix(Rule::equal, Assoc::Left) | Op::infix(Rule::not_equal, Assoc::Left))
            .op(Op::infix(Rule::greater_than, Assoc::Left)
                | Op::infix(Rule::greater_equal, Assoc::Left)
                | Op::infix(Rule::lesser_than, Assoc::Left)
                | Op::infix(Rule::lesser_equal, Assoc::Left))
            .op(Op::prefix(Rule::not))
    });

    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::expr => parse_expr(primary.into_inner()),
            Rule::ident => Expr::Ident(primary.as_str()),
            Rule::number => Expr::Value(parse_number(primary.as_str())),
            other => unreachable!("{:?}", other),
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::not => Expr::Unop {
                op: UnopType::Not,
                expr: Box::new(rhs),
            },
            other => unreachable!("{:?}", other),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::equal => BinopType::Eq,
                Rule::not_equal => BinopType::Ne,
                Rule::or => BinopType::Or,
                Rule::and => BinopType::And,
                Rule::greater_than => BinopType::Gt,
                Rule::greater_equal => BinopType::Ge,
                Rule::lesser_than => BinopType::Lt,
                Rule::lesser_equal => BinopType::Le,
                other => unreachable!("{:?}", other),
            };

            Expr::Binop {
                left: Box::new(lhs),
                op,
                right: Box::new(rhs),
            }
        })
        .parse(pairs)
}

fn parse_number(s: &str) -> u64 {
    if s.starts_with("0x") || s.starts_with("0X") {
        return u64::from_str_radix(&s[2..], 16).expect("Failed to parse number");
    }

    if s.starts_with("0b") || s.starts_with("0B") {
        return u64::from_str_radix(&s[2..], 2).expect("Failed to parse number");
    }

    u64::from_str(s).expect("Failed to parse number")
}

fn parse_range(pair: Pair<Rule>) -> KeywordRange {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::range_inclusive => {
            let mut range_inner = inner.into_inner();

            KeywordRange::Inclusive(RangeInclusive {
                start: parse_number(range_inner.next().unwrap().as_str()),
                last: parse_number(range_inner.next().unwrap().as_str()),
            })
        }
        Rule::range_exclusive => {
            let mut range_inner = inner.into_inner();

            KeywordRange::Exclusive(Range {
                start: parse_number(range_inner.next().unwrap().as_str()),
                end: parse_number(range_inner.next().unwrap().as_str()),
            })
        }
        _ => unreachable!(),
    }
}
