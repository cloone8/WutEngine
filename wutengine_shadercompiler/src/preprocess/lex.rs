use logos::Logos;

use super::parse::{Condition, ParseMacroErr};

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"\s+")]
pub(super) enum MacroToken<'a> {
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

    #[regex("[0-9]+")]
    DecInt(&'a str),

    #[regex("[A-Z_][A-Z_0-9]*")]
    Keyword(&'a str),
}

impl MacroToken<'_> {
    pub(super) fn as_condition(&self) -> Result<Condition, ParseMacroErr> {
        Ok(match self {
            MacroToken::Eq => Condition::Eq,
            MacroToken::Ne => Condition::Ne,
            MacroToken::Lt => Condition::Lt,
            MacroToken::Gt => Condition::Gt,
            MacroToken::Le => Condition::Le,
            MacroToken::Ge => Condition::Ge,
            other => {
                return Err(ParseMacroErr::UnexpectedToken {
                    expected: "<a condition>".to_string(),
                    actual: format!("{:#?}", other),
                });
            }
        })
    }

    pub(super) fn as_int(&self) -> Result<u32, ParseMacroErr> {
        Ok(match self {
            MacroToken::HexInt(hex) => u32::from_str_radix(hex, 16)?,
            MacroToken::BinInt(bin) => u32::from_str_radix(bin, 2)?,
            MacroToken::DecInt(dec) => dec.parse::<u32>()?,
            other => {
                return Err(ParseMacroErr::UnexpectedToken {
                    expected: "<an integer>".to_string(),
                    actual: format!("{:#?}", other),
                });
            }
        })
    }
}
