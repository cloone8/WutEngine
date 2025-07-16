use core::num::ParseIntError;
use core::str::FromStr;

use logos::Logos;
use thiserror::Error;

/// Prefix tokens used to start WutEngine shader preprocessor directives, e.g. `//!! IF A == 5`
pub(crate) const DIRECTIVE_PREFIX: &str = "//!!";

#[derive(Debug)]
pub(super) enum RawStatement {
    Directive(String),
    Code(String),
}

impl RawStatement {
    pub(super) fn parse(self) -> Result<Statement, ParseDirectiveErr> {
        Ok(match self {
            RawStatement::Directive(macro_str) => {
                Statement::Directive(Directive::from_str(&macro_str)?)
            }
            RawStatement::Code(code) => Statement::Code(code),
        })
    }
}

#[derive(Debug)]
pub(super) enum Statement {
    Directive(Directive),
    Code(String),
}

#[derive(Debug, Error)]
pub enum ParseDirectiveErr {
    #[error("Unrecognized token")]
    Lexer,

    #[error("Empty macro")]
    Empty,

    #[error("Unknown macro: {}", .0)]
    UnknownDirective(String),

    #[error("Too many tokens: {}", .0)]
    DanglingTokens(String),

    #[error("Unfinished macro")]
    MissingToken,

    #[error("Unexpected token. Expected {}, got {}", .expected, .actual)]
    UnexpectedToken { expected: String, actual: String },

    #[error("Could not parse integer")]
    UnknownInt(#[from] ParseIntError),
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"\s+")]
pub(super) enum DirectiveToken<'a> {
    #[token("IF")]
    If,

    #[token("ELSE")]
    Else,

    #[token("ENDIF")]
    Endif,

    #[token("==")]
    Eq,

    #[token("!=")]
    Ne,

    #[token("<")]
    Lt,

    #[token(">")]
    Gt,

    #[token("<=")]
    Le,

    #[token(">=")]
    Ge,

    #[regex("0x[abcdef0-9]+")]
    HexInt(&'a str),

    #[regex("0b[10]+")]
    BinInt(&'a str),

    #[regex("-?[0-9]+")]
    DecInt(&'a str),

    #[regex("[A-Z_][A-Z_0-9]*")]
    Keyword(&'a str),
}

impl DirectiveToken<'_> {
    pub(super) fn as_condition(&self) -> Result<Condition, ParseDirectiveErr> {
        Ok(match self {
            DirectiveToken::Eq => Condition::Eq,
            DirectiveToken::Ne => Condition::Ne,
            DirectiveToken::Lt => Condition::Lt,
            DirectiveToken::Gt => Condition::Gt,
            DirectiveToken::Le => Condition::Le,
            DirectiveToken::Ge => Condition::Ge,
            other => {
                return Err(ParseDirectiveErr::UnexpectedToken {
                    expected: "<a condition>".to_string(),
                    actual: format!("{other:#?}"),
                });
            }
        })
    }

    pub(super) fn as_int(&self) -> Result<i64, ParseDirectiveErr> {
        Ok(match self {
            DirectiveToken::HexInt(hex) => u64::from_str_radix(hex, 16)? as i64,
            DirectiveToken::BinInt(bin) => u64::from_str_radix(bin, 2)? as i64,
            DirectiveToken::DecInt(dec) => dec.parse::<i64>()?,
            other => {
                return Err(ParseDirectiveErr::UnexpectedToken {
                    expected: "<an integer>".to_string(),
                    actual: format!("{other:#?}"),
                });
            }
        })
    }
}

#[derive(Debug, Clone)]
pub(super) enum Directive {
    If {
        keyword: String,
        condition: Condition,
        value: i64,
    },
    Else,
    Endif,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Condition {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

impl FromStr for Directive {
    type Err = ParseDirectiveErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lexer = DirectiveToken::lexer(s);

        let mut tokens = lexer
            .collect::<Result<Vec<_>, ()>>()
            .map_err(|_| ParseDirectiveErr::Lexer)?;

        log::debug!("Tokens: {tokens:#?}");

        if tokens.is_empty() {
            return Err(ParseDirectiveErr::Empty);
        }

        tokens.reverse();

        let parsed = match tokens.pop().unwrap() {
            DirectiveToken::If => parse_if(&mut tokens)?,
            DirectiveToken::Else => Directive::Else,
            DirectiveToken::Endif => Directive::Endif,
            other => return Err(ParseDirectiveErr::UnknownDirective(format!("{other:#?}"))),
        };

        if !tokens.is_empty() {
            return Err(ParseDirectiveErr::DanglingTokens(format!("{tokens:#?}")));
        }

        log::debug!("Parsed: {parsed:#?}");

        Ok(parsed)
    }
}

fn parse_if(tokens: &mut Vec<DirectiveToken>) -> Result<Directive, ParseDirectiveErr> {
    let keyword = tokens.pop().ok_or(ParseDirectiveErr::MissingToken)?;

    let keyword = if let DirectiveToken::Keyword(kw) = keyword {
        kw.to_string()
    } else {
        return Err(ParseDirectiveErr::UnexpectedToken {
            expected: "<a keyword>".to_string(),
            actual: format!("{keyword:#?}"),
        });
    };

    let condition = tokens
        .pop()
        .ok_or(ParseDirectiveErr::MissingToken)?
        .as_condition()?;

    let value = tokens
        .pop()
        .ok_or(ParseDirectiveErr::MissingToken)?
        .as_int()?;

    Ok(Directive::If {
        keyword,
        condition,
        value,
    })
}
