use core::borrow::Borrow;
use core::hash::Hash;
use std::collections::HashMap;

const DIRECTIVE_LEADER: &str = "//#";

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum ParseErr {
    #[display("Unknown compiler directive given: {}", _0)]
    InvalidDirective(#[error(not(source))] String),

    #[display("Malformed condition string \"{}\", error: {}", condition, reason)]
    MalformedCondition {
        condition: String,
        reason: &'static str,
    },

    #[display("Unmatched opening brace found")]
    UnmatchedOpeningBrace,

    #[display("Unmatched closing brace found")]
    UnmatchedClosingBrace,

    #[display("Invalid condition comparator given: {}", _0)]
    InvalidConditionComparator(#[error(not(source))] String),
}

#[derive(Debug, Clone)]
pub struct ShaderFile<'a>(pub Vec<Statement<'a>>);

impl<'src> ShaderFile<'src> {
    pub fn parse(source: &'src str) -> Result<Self, Box<ParseErr>> {
        Ok(Self(
            source
                .lines()
                .map(Statement::parse)
                .collect::<Result<Vec<_>, Box<ParseErr>>>()?,
        ))
    }
}

#[derive(Debug, Clone)]
pub enum Statement<'a> {
    Source(&'a str),
    Directive(Directive<'a>),
}

impl<'src> Statement<'src> {
    pub fn parse(source: &'src str) -> Result<Self, Box<ParseErr>> {
        let trimmed = source.trim();

        if trimmed.starts_with(DIRECTIVE_LEADER) {
            let without_leader = trimmed.strip_prefix(DIRECTIVE_LEADER).unwrap();

            Ok(Self::Directive(Directive::parse(without_leader)?))
        } else {
            Ok(Self::Source(source)) // Push source with whitespace because it looks nicer
        }
    }
}

#[derive(Debug, Clone)]
pub enum Directive<'a> {
    If(Condition<'a>),
    Elif(Condition<'a>),
    Else,
    Endif,
}

impl<'src> Directive<'src> {
    pub fn parse(source: &'src str) -> Result<Self, Box<ParseErr>> {
        let trimmed = source.trim();

        if trimmed.starts_with("if") {
            let no_prefix = trimmed.strip_prefix("if").unwrap();

            let condition = Condition::parse(no_prefix)?;

            Ok(Self::If(condition))
        } else if trimmed.starts_with("else if") {
            let no_prefix = trimmed.strip_prefix("else if").unwrap();

            let condition = Condition::parse(no_prefix)?;

            Ok(Self::Elif(condition))
        } else if trimmed == "else" {
            Ok(Self::Else)
        } else if trimmed == "endif" {
            Ok(Self::Endif)
        } else {
            Err(Box::new(ParseErr::InvalidDirective(trimmed.to_string())))
        }
    }
}

#[derive(Debug, Clone)]
pub enum Condition<'a> {
    Single {
        keyword: &'a str,
        comparator: ConditionComparator,
        value: u64,
    },
    Chain {
        left: Box<Condition<'a>>,
        op: ConditionChain,
        right: Box<Condition<'a>>,
    },
}

impl<'src> Condition<'src> {
    fn parse(source: &'src str) -> Result<Self, Box<ParseErr>> {
        let s = source.trim();

        // strip outer parens if they enclose whole expression
        if s.starts_with('(') && Self::matching_paren(s)? == (s.len() - 1) {
            return Self::parse(&s[1..s.len() - 1]);
        }

        // lowest precedence first: ||
        if let Some(i) = Self::find_top_level(s, "||") {
            return Ok(Self::Chain {
                left: Box::new(Self::parse(&s[..i])?),
                op: ConditionChain::Or,
                right: Box::new(Self::parse(&s[i + 2..])?),
            });
        }

        // then &&
        if let Some(i) = Self::find_top_level(s, "&&") {
            return Ok(Self::Chain {
                left: Box::new(Self::parse(&s[..i])?),
                op: ConditionChain::And,
                right: Box::new(Self::parse(&s[i + 2..])?),
            });
        }

        // must be a condition: IDENT OP NUMBER
        let mut parts = s.split_whitespace();

        let ident = parts.next().ok_or_else(|| ParseErr::MalformedCondition {
            condition: s.to_string(),
            reason: "Missing or malformed identifier",
        })?;

        let op = parts.next().ok_or_else(|| ParseErr::MalformedCondition {
            condition: s.to_string(),
            reason: "Missing or malformed operator",
        })?;

        let value = parts
            .next()
            .ok_or_else(|| ParseErr::MalformedCondition {
                condition: s.to_string(),
                reason: "Missing or malformed operator comparison value",
            })?
            .parse()
            .unwrap();

        if !ident.chars().all(|c| c.is_ascii_alphabetic() || c == '_') {
            return Err(Box::new(ParseErr::MalformedCondition {
                condition: s.to_string(),
                reason: "Identifier contains invalid characters. Only a-zA-Z and _ are allowed.",
            }));
        }

        Ok(Self::Single {
            keyword: ident,
            comparator: ConditionComparator::parse(op)?,
            value,
        })
    }

    fn find_top_level(s: &str, pat: &str) -> Option<usize> {
        let mut depth = 0;
        let bytes = s.as_bytes();
        let pat_b = pat.as_bytes();

        for i in 0..s.len().saturating_sub(1) {
            match bytes[i] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            if depth == 0 && &bytes[i..i + 2] == pat_b {
                return Some(i);
            }
        }
        None
    }

    fn matching_paren(s: &str) -> Result<usize, Box<ParseErr>> {
        let mut depth = 0;
        for (i, c) in s.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;

                    if depth < 0 {
                        return Err(Box::new(ParseErr::UnmatchedClosingBrace));
                    }

                    if depth == 0 {
                        return Ok(i);
                    }
                }
                _ => {}
            }
        }

        Err(Box::new(ParseErr::UnmatchedOpeningBrace))
    }

    pub fn eval<'a, S: AsRef<str> + Hash + Eq + Borrow<str>>(
        &'a self,
        keywords: &HashMap<S, u64>,
    ) -> Result<bool, &'a str> {
        match self {
            Self::Single {
                keyword,
                comparator,
                value,
            } => {
                let condition_val = *value;
                let kw = *keyword;
                let actual_val = *keywords.get(kw).ok_or(kw)?;

                let result = match comparator {
                    ConditionComparator::Eq => actual_val == condition_val,
                    ConditionComparator::Ne => actual_val != condition_val,
                    ConditionComparator::Lt => actual_val < condition_val,
                    ConditionComparator::Le => actual_val <= condition_val,
                    ConditionComparator::Gt => actual_val > condition_val,
                    ConditionComparator::Ge => actual_val >= condition_val,
                };

                Ok(result)
            }
            Self::Chain { left, op, right } => match op {
                ConditionChain::And => Ok(left.eval(keywords)? && right.eval(keywords)?),
                ConditionChain::Or => Ok(left.eval(keywords)? || right.eval(keywords)?),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConditionComparator {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl ConditionComparator {
    fn parse(source: &str) -> Result<Self, Box<ParseErr>> {
        Ok(match source.trim() {
            "==" => Self::Eq,
            "!=" => Self::Ne,
            "<" => Self::Lt,
            "<=" => Self::Le,
            ">" => Self::Gt,
            ">=" => Self::Ge,
            _ => {
                return Err(Box::new(ParseErr::InvalidConditionComparator(
                    source.to_owned(),
                )));
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConditionChain {
    And,
    Or,
}

pub fn parse_condition(s: &str) -> Result<Condition<'_>, Box<ParseErr>> {
    Condition::parse(s)
}
