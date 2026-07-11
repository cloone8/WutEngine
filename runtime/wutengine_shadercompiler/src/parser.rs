//! WutEngine shader compiler directive parser

use core::{borrow::Borrow, hash::Hash};

use alloc::{
    borrow::ToOwned,
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use hashbrown::HashMap;

const DIRECTIVE_LEADER: &str = "//#";

/// An error while parsing a WutEngine shader source file
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum ParseErr {
    /// An unknown directive
    #[display("Unknown compiler directive given: {}", _0)]
    InvalidDirective(#[error(not(source))] String),

    /// Malformed condition string
    #[display("Malformed condition string \"{}\", error: {}", condition, reason)]
    MalformedCondition {
        /// The condition string
        condition: String,

        /// The reason it was not able to be parsed
        reason: &'static str,
    },

    /// Unmatched opening brace (`(`)
    #[display("Unmatched opening brace found")]
    UnmatchedOpeningBrace,

    /// Unmatched closing brace (`)`)
    #[display("Unmatched closing brace found")]
    UnmatchedClosingBrace,

    /// Invalid condition comparator (`==`, `!=`, etc.)
    #[display("Invalid condition comparator given: {}", _0)]
    InvalidConditionComparator(#[error(not(source))] String),
}

/// A parsed shader source file
#[derive(Debug, Clone)]
pub(crate) struct ShaderFile<'a>(pub Vec<Statement<'a>>);

impl<'src> ShaderFile<'src> {
    /// Attempts to parse the string into a source file
    pub(crate) fn parse(source: &'src str) -> Result<Self, Box<ParseErr>> {
        profiling::function_scope!();

        log::debug!("Parsing shader file");

        Ok(Self(
            source
                .lines()
                .map(Statement::parse)
                .collect::<Result<Vec<_>, Box<ParseErr>>>()?,
        ))
    }
}

/// A single statement in a [ShaderFile]
#[derive(Debug, Clone)]
pub(crate) enum Statement<'a> {
    /// Normal source code
    Source(&'a str),

    /// A preprocessor directive
    Directive(Directive<'a>),
}

impl<'src> Statement<'src> {
    fn parse(source: &'src str) -> Result<Self, Box<ParseErr>> {
        let trimmed = source.trim();

        if trimmed.starts_with(DIRECTIVE_LEADER) {
            let without_leader = trimmed.strip_prefix(DIRECTIVE_LEADER).unwrap();

            Ok(Self::Directive(Directive::parse(without_leader)?))
        } else {
            Ok(Self::Source(source)) // Push source with whitespace because it looks nicer
        }
    }
}

/// A preprocessor directive in a [ShaderFile]
#[derive(Debug, Clone)]
pub(crate) enum Directive<'a> {
    /// `if` with the condition
    If(Condition<'a>),
    /// `elif` with the condition
    Elif(Condition<'a>),

    /// `else`
    Else,

    /// `endif`
    Endif,
}

impl<'src> Directive<'src> {
    fn parse(source: &'src str) -> Result<Self, Box<ParseErr>> {
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

/// A condition that can be evaluated
#[derive(Debug, Clone)]
pub(crate) enum Condition<'a> {
    /// A non-chained condition (`if FOO == 5`)
    Single {
        /// The keyword to compare
        keyword: &'a str,
        /// The comparator
        comparator: ConditionComparator,

        /// The value to compare against
        value: u64,
    },

    /// A chained condition (possibly nested) (`if FOO == 5 && BAR == 6`)
    Chain {
        /// The left inner condition
        left: Box<Condition<'a>>,

        /// The chaining operator (`&&`, `||`, etc.)
        op: ConditionChain,

        /// The right inner condition
        right: Box<Condition<'a>>,
    },
}

impl<'src> Condition<'src> {
    /// Parses the given source string into a condition, if possible.
    ///
    /// Requires `source` to be pre-stripped of both the [directive leader](DIRECTIVE_LEADER) and the [Directive] (`if`, `elif`, etc.)
    pub(crate) fn parse(source: &'src str) -> Result<Self, Box<ParseErr>> {
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

    /// Evaluates whether the condition is true or false, based on the given input
    /// keywords.
    ///
    /// If the condition could not be fully evaluated due to a missing keyword value,
    /// returns [Err] with the keyword that was missing
    pub(crate) fn eval<'a, S: AsRef<str> + Hash + Eq + Borrow<str>>(
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

/// A comparator for a [Condition]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ConditionComparator {
    /// `==`
    Eq,

    /// `!=`
    Ne,

    /// `<`
    Lt,

    /// `<=`
    Le,

    /// `>`
    Gt,

    /// `>=`
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

/// An operator to chain multiple [Conditions](Condition)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ConditionChain {
    /// `&&`
    And,

    /// `||`
    Or,
}
