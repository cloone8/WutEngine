//! Shader preprocessing

use core::error::Error;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::ShaderResolver;
use crate::preprocessor::parser::Expr;
use crate::preprocessor::parser::Rule;

mod parser;

/// Start token for a preprocessor directive. Lines that start with this character will be parsed as a preprocessor directive
const DIRECTIVE_START: &str = "#";

/// An error during shader preprocessing
#[derive(Debug, derive_more::Error, derive_more::Display, derive_more::From)]
pub enum PreprocessErr {
    /// Resolver failed
    #[display("Shader resolver returned an error: {_0}")]
    #[from(skip)]
    Resolver(Box<dyn Error>),

    /// Directive could not be parsed
    #[display("Error parsing a directive: {_0}")]
    DirectiveParser(Box<pest::error::Error<Rule>>),

    /// Branch mismatch
    #[display("if/else mismatch")]
    BranchMismatch,
}

struct PreprocessorState<'a, S> {
    keywords: HashMap<String, u64, S>,
    included_files: HashSet<String>,
    resolver: &'a dyn ShaderResolver,
}

impl<S> PreprocessorState<'_, S>
where
    S: ::core::hash::BuildHasher,
{
    fn preprocess(&mut self, input: &str) -> Result<String, PreprocessErr> {
        let mut output = String::new();

        let mut branch_stack: Vec<bool> = Vec::with_capacity(32);

        for line in input.lines() {
            if !line.trim().starts_with(DIRECTIVE_START) {
                // Source line
                if Self::branch_stack_active(&branch_stack) {
                    output.push_str(&self.substitute_keywords(line));
                    output.push('\n');
                }
                continue;
            }

            let directive = parser::parse_directive(line)
                .map_err(|e| PreprocessErr::DirectiveParser(Box::new(e)))?;

            match directive {
                parser::Directive::Name(_) => {
                    // Not relevant here
                }
                parser::Directive::KeywordDecl { .. } => {
                    // Not relevant here
                }
                parser::Directive::Import(name) => {
                    if !Self::branch_stack_active(&branch_stack) {
                        continue;
                    }

                    if self.included_files.contains(name) {
                        // Already imported once
                        continue;
                    }

                    self.included_files.insert(name.to_string());

                    log::debug!("Importing shader `{name}`");

                    let imported_content = self
                        .resolver
                        .find_by_name(name)
                        .map_err(PreprocessErr::Resolver)?;

                    let result = self.preprocess(&imported_content)?;

                    output.push_str(&result);
                }
                parser::Directive::If(expr) => {
                    branch_stack.push(
                        Self::branch_stack_active(&branch_stack)
                            && (self.evaluate_expr(&expr) != 0),
                    );
                }
                parser::Directive::Else => {
                    let prev_val = branch_stack.pop().ok_or(PreprocessErr::BranchMismatch)?;
                    branch_stack.push(!prev_val);
                }
                parser::Directive::Endif => {
                    _ = branch_stack.pop().ok_or(PreprocessErr::BranchMismatch)?;
                }
            }
        }

        if !branch_stack.is_empty() {
            return Err(PreprocessErr::BranchMismatch);
        }

        Ok(output)
    }

    fn branch_stack_active(branch_stack: &[bool]) -> bool {
        branch_stack.iter().copied().all(|x| x)
    }

    fn evaluate_expr(&self, expr: &Expr) -> u64 {
        match expr {
            Expr::Ident(ident) => self.keywords.get(*ident).copied().unwrap_or(0),
            Expr::Value(val) => *val,
            Expr::Unop { op, expr: subexpr } => match op {
                parser::UnopType::Not => u64::from(self.evaluate_expr(subexpr) == 0),
            },
            Expr::Binop { left, op, right } => match op {
                parser::BinopType::Eq => {
                    u64::from(self.evaluate_expr(left) == self.evaluate_expr(right))
                }
                parser::BinopType::Ne => {
                    u64::from(self.evaluate_expr(left) != self.evaluate_expr(right))
                }
                parser::BinopType::Gt => {
                    u64::from(self.evaluate_expr(left) > self.evaluate_expr(right))
                }
                parser::BinopType::Ge => {
                    u64::from(self.evaluate_expr(left) >= self.evaluate_expr(right))
                }
                parser::BinopType::Lt => {
                    u64::from(self.evaluate_expr(left) < self.evaluate_expr(right))
                }
                parser::BinopType::Le => {
                    u64::from(self.evaluate_expr(left) <= self.evaluate_expr(right))
                }
                parser::BinopType::And => {
                    if self.evaluate_expr(left) != 0 {
                        self.evaluate_expr(right)
                    } else {
                        0
                    }
                }
                parser::BinopType::Or => {
                    let left_val = self.evaluate_expr(left);

                    if left_val != 0 {
                        left_val
                    } else {
                        self.evaluate_expr(right)
                    }
                }
            },
        }
    }

    /// Searches for keyword definitions inside the given source code line, and replaces
    /// them with the current value of the keyword
    fn substitute_keywords<'a>(&self, line: &'a str) -> Cow<'a, str> {
        let mut ret = Cow::Borrowed(line);

        for (keyword, value) in &self.keywords {
            let keyword_str = keyword.as_str();

            if let Some(start_index) = ret.find(keyword_str) {
                let keyword_byte_range = start_index..(start_index + keyword_str.len());

                Cow::to_mut(&mut ret)
                    .replace_range(keyword_byte_range, format!("{value}").as_str());
            }
        }

        ret
    }
}

/// Preprocesses the shader source using the provided keyword values and shader resolver
pub fn preprocess<S: ::core::hash::BuildHasher>(
    input: &str,
    keywords: HashMap<String, u64, S>,
    shader_resolver: &dyn ShaderResolver,
) -> Result<String, PreprocessErr> {
    log::debug!("Preprocessing shader");

    let mut state = PreprocessorState {
        keywords,
        included_files: HashSet::new(),
        resolver: shader_resolver,
    };

    state.preprocess(input)
}
