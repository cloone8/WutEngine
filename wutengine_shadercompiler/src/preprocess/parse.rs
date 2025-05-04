use core::num::ParseIntError;
use core::str::FromStr;

use logos::Logos;
use thiserror::Error;

use crate::preprocess::lex::MacroToken;

#[derive(Debug, Error)]
pub enum ParseMacroErr {
    #[error("Unrecognized token")]
    Lexer,

    #[error("Empty macro")]
    Empty,

    #[error("Unknown macro: {}", .0)]
    UnknownMacro(String),

    #[error("Too many tokens: {}", .0)]
    DanglingTokens(String),

    #[error("Unfinished macro")]
    MissingToken,

    #[error("Unexpected token. Expected {}, got {}", .expected, .actual)]
    UnexpectedToken { expected: String, actual: String },

    #[error("Could not parse integer")]
    UnknownInt(#[from] ParseIntError),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub(super) enum Macro {
    If {
        keyword: String,
        condition: Condition,
        value: u32,
    },
    Else,
    Endif,
}

impl FromStr for Macro {
    type Err = ParseMacroErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lexer = MacroToken::lexer(s);

        let mut tokens = lexer
            .collect::<Result<Vec<_>, ()>>()
            .map_err(|_| ParseMacroErr::Lexer)?;

        log::debug!("Tokens: {:#?}", tokens);

        if tokens.is_empty() {
            return Err(ParseMacroErr::Empty);
        }

        tokens.reverse();

        let parsed = match tokens.pop().unwrap() {
            MacroToken::If => parse_if(&mut tokens)?,
            MacroToken::Else => Macro::Else,
            MacroToken::Endif => Macro::Endif,
            other => return Err(ParseMacroErr::UnknownMacro(format!("{:#?}", other))),
        };

        if !tokens.is_empty() {
            return Err(ParseMacroErr::DanglingTokens(format!("{:#?}", tokens)));
        }

        log::debug!("Parsed: {:#?}", parsed);

        Ok(parsed)
    }
}

fn parse_if(tokens: &mut Vec<MacroToken>) -> Result<Macro, ParseMacroErr> {
    let keyword = tokens.pop().ok_or(ParseMacroErr::MissingToken)?;

    let keyword = if let MacroToken::Keyword(kw) = keyword {
        kw.to_string()
    } else {
        return Err(ParseMacroErr::UnexpectedToken {
            expected: "<a keyword>".to_string(),
            actual: format!("{:#?}", keyword),
        });
    };

    let condition = tokens
        .pop()
        .ok_or(ParseMacroErr::MissingToken)?
        .as_condition()?;

    let value = tokens.pop().ok_or(ParseMacroErr::MissingToken)?.as_int()?;

    Ok(Macro::If {
        keyword,
        condition,
        value,
    })
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
