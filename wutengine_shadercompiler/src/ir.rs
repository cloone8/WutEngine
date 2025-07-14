use core::num::ParseIntError;
use core::str::FromStr;
use std::collections::HashMap;

use logos::Logos;
use thiserror::Error;

pub(crate) const DIRECTIVE_PREFIX: &str = "//!!";

#[derive(Debug)]
pub(crate) enum RawStatement {
    Directive(String),
    Code(String),
}

impl RawStatement {
    fn parse(self) -> Result<Statement, ParseDirectiveErr> {
        Ok(match self {
            RawStatement::Directive(macro_str) => {
                Statement::Directive(Directive::from_str(&macro_str)?)
            }
            RawStatement::Code(code) => Statement::Code(code),
        })
    }
}

#[derive(Debug)]
pub(crate) enum Statement {
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

#[derive(Debug, Error)]
pub enum PreprocessErr {
    #[error("Parsing macro failed: {}", .0)]
    ParseDirective(#[from] ParseDirectiveErr),

    #[error("Unexpected macro at this point in the source code: {}", .0)]
    UnexpectedDirective(String),

    #[error("Directive IF/ENDIF mismatch")]
    MissingEndif,
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
                    actual: format!("{:#?}", other),
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
                    actual: format!("{:#?}", other),
                });
            }
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Directive {
    If {
        keyword: String,
        condition: Condition,
        value: i64,
    },
    Else,
    Endif,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Condition {
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

        log::debug!("Tokens: {:#?}", tokens);

        if tokens.is_empty() {
            return Err(ParseDirectiveErr::Empty);
        }

        tokens.reverse();

        let parsed = match tokens.pop().unwrap() {
            DirectiveToken::If => parse_if(&mut tokens)?,
            DirectiveToken::Else => Directive::Else,
            DirectiveToken::Endif => Directive::Endif,
            other => return Err(ParseDirectiveErr::UnknownDirective(format!("{:#?}", other))),
        };

        if !tokens.is_empty() {
            return Err(ParseDirectiveErr::DanglingTokens(format!("{:#?}", tokens)));
        }

        log::debug!("Parsed: {:#?}", parsed);

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
            actual: format!("{:#?}", keyword),
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

fn split_blocks(source: &str) -> Vec<RawStatement> {
    let mut blocks = Vec::new();

    let mut cur_source = Vec::new();

    for line in source.lines() {
        if line.trim().starts_with(DIRECTIVE_PREFIX) {
            if !cur_source.is_empty() {
                let code_block = cur_source.join("\n");
                blocks.push(RawStatement::Code(code_block));
                cur_source.clear();
            }

            blocks.push(RawStatement::Directive(
                line.trim()
                    .strip_prefix(DIRECTIVE_PREFIX)
                    .unwrap()
                    .to_string(),
            ));
        } else {
            cur_source.push(line.to_string());
        }
    }

    if !cur_source.is_empty() {
        let code_block = cur_source.join("\n");
        blocks.push(RawStatement::Code(code_block));
    }

    blocks
}

pub(crate) fn preprocess(
    raw_shader: &str,
    keywords: &HashMap<&str, i64>,
) -> Result<String, PreprocessErr> {
    // Split the source into lines. We then join the lines into either a "macro" line, or multiple "code" lines

    let mut blocks = split_blocks(raw_shader)
        .into_iter()
        .map(RawStatement::parse)
        .collect::<Result<Vec<_>, ParseDirectiveErr>>()?;

    log::debug!("Blocks: {:#?}", blocks);

    while blocks.iter().any(|b| matches!(b, Statement::Directive(_))) {
        let (next_macro_idx, next_macro) = blocks
            .iter()
            .enumerate()
            .find_map(|(i, b)| match b {
                Statement::Directive(m) => Some((i, m)),
                Statement::Code(_) => None,
            })
            .unwrap();

        match next_macro {
            Directive::If {
                keyword,
                condition,
                value,
            } => {
                let kw_val = keywords.get(keyword.as_str()).unwrap_or(&0);

                let is_true = match condition {
                    Condition::Eq => kw_val == value,
                    Condition::Ne => kw_val != value,
                    Condition::Lt => kw_val < value,
                    Condition::Gt => kw_val > value,
                    Condition::Le => kw_val <= value,
                    Condition::Ge => kw_val >= value,
                };

                process_branch(&mut blocks, next_macro_idx + 1, is_true)?;
            }
            other => return Err(PreprocessErr::UnexpectedDirective(format!("{:#?}", other))),
        }

        blocks.remove(next_macro_idx);

        log::debug!("Post-stage {:#?}", blocks);
    }

    let mut code = blocks
        .into_iter()
        .map(|block| match block {
            Statement::Directive(_) => unreachable!("All macros should have been processed"),
            Statement::Code(c) => c,
        })
        .collect::<Vec<_>>()
        .join("\n");

    for (&keyword, val) in keywords {
        code = code.replace(keyword, format!("{val}").as_str());
    }

    Ok(code)
}

fn process_branch(
    blocks: &mut Vec<Statement>,
    branch_start: usize,
    is_true: bool,
) -> Result<(), PreprocessErr> {
    let mut branch_stack = 0;
    let mut else_start = None;
    let mut branch_end = None;

    for (i, block) in blocks.iter().enumerate().skip(branch_start) {
        match block {
            Statement::Directive(Directive::If { .. }) => {
                branch_stack += 1;
            }
            Statement::Directive(Directive::Else) => {
                if branch_stack == 0 {
                    else_start = Some(i);
                }
            }
            Statement::Directive(Directive::Endif) => {
                if branch_stack == 0 {
                    branch_end = Some(i);
                    break;
                } else {
                    branch_stack -= 1;
                }
            }
            Statement::Code(_) => continue,
        }
    }

    log::debug!(
        "if {} else {:?} end {:?}",
        branch_start - 1,
        else_start,
        branch_end
    );

    let branch_end = match branch_end {
        Some(be) => be,
        None => {
            return Err(PreprocessErr::MissingEndif);
        }
    };

    match is_true {
        true => {
            // Remove else -> endif if there is an `else`
            // Otherwise just remove the endif macro
            match else_start {
                Some(else_start) => {
                    let diff = branch_end - else_start;

                    for _ in 0..(diff + 1) {
                        blocks.remove(else_start);
                    }
                }
                None => {
                    blocks.remove(branch_end);
                }
            }
        }
        false => {
            // Remove if -> else and endif if there is an `else`
            // Otherwise just remove everything
            match else_start {
                Some(else_start) => {
                    let diff = else_start - branch_start;

                    blocks.remove(branch_end);
                    for _ in 0..(diff + 1) {
                        blocks.remove(branch_start);
                    }
                }
                None => {
                    let diff = branch_end - branch_start;

                    for _ in 0..(diff + 1) {
                        blocks.remove(branch_start);
                    }
                }
            }
        }
    }

    Ok(())
}
