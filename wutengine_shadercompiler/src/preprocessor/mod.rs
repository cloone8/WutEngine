//! Preprocessor module for WutEngine WGSL shaders. Processes C-like directives with a slightly different syntax,
//! to support things like conditional compilation and multi-shader-variants

use std::collections::HashMap;

use itertools::Itertools;
use thiserror::Error;

use crate::ShaderOutput;
use crate::preprocessor::ir::{
    Condition, DIRECTIVE_PREFIX, Directive, ParseDirectiveErr, RawStatement, Statement,
};

mod ir;

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

/// Error during preprocessing
#[derive(Debug, Error)]
pub enum PreprocessErr {
    /// An unknown directive was encountered
    #[error("Parsing macro failed: {}", .0)]
    ParseDirective(#[from] ParseDirectiveErr),

    /// This directive was not expected at this part of the source code
    #[error("Unexpected macro at this point in the source code: {}", .0)]
    UnexpectedDirective(String),

    /// Missing endif directive after if directive
    #[error("Directive IF/ENDIF mismatch")]
    MissingEndif,
}

/// Preprocesses the given raw shader into a [ShaderOutput::Preprocessed] shader, applying the given keyword values.
/// Any unknown keywords in directives are given value `0`
#[profiling::function]
pub(crate) fn preprocess<'a>(
    raw_shader: &str,
    keywords: &HashMap<&'a str, i64>,
) -> Result<ShaderOutput<'a>, PreprocessErr> {
    // Split the source into lines. We then join the lines into either a "macro" line, or multiple "code" lines

    let mut blocks = split_blocks(raw_shader)
        .into_iter()
        .map(RawStatement::parse)
        .collect::<Result<Vec<_>, ParseDirectiveErr>>()?;

    log::debug!("Blocks: {blocks:#?}");

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
            other => return Err(PreprocessErr::UnexpectedDirective(format!("{other:#?}"))),
        }

        blocks.remove(next_macro_idx);

        log::debug!("Post-stage {blocks:#?}");
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

    let keyword_meta = keywords
        .iter()
        .map(|(k, v)| format!("// {k}={v}"))
        .join("\n");
    code = format!("{code}\n\n// WUTENGINE SHADER COMPILER KEYWORDS:\n//\n{keyword_meta}\n");

    Ok(ShaderOutput::Preprocessed {
        source: code,
        keyword_hash: wutengine_util::hash::keyword_hash(keywords),
        keywords: keywords.clone(),
    })
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
