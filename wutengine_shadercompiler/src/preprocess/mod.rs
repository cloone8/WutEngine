//! Shader preprocessor

use core::str::FromStr;
use std::collections::HashMap;

use parse::{Condition, Macro, ParseMacroErr};
use thiserror::Error;
use wutengine_graphics::shader::{RawShader, ShaderStage};

mod lex;
mod parse;

#[derive(Debug, Error)]
pub enum PreprocessErr {
    #[error("Parsing macro failed: {}", .0)]
    ParseMacro(#[from] ParseMacroErr),

    #[error("Unexpected macro at this point in the source code: {}", .0)]
    UnexpectedMacro(String),

    #[error("Macro IF/ENDIF mismatch")]
    MissingEndif,
}

/// Macro for the WutEngine macro prefix, so that we can use it with [concat!]
macro_rules! macro_prefix {
    () => {
        "//!!"
    };
}

/// Preprocesses the given shader by expanding macros and resolving active macro branches
/// according to the given keywords.
/// Keywords in the source code that aren't given in `keywords` are presumed to be `0`
pub(crate) fn preprocess(
    shader: &mut RawShader,
    keywords: &HashMap<String, u32>,
) -> Result<(), PreprocessErr> {
    if let Some(vertex) = &mut shader.source.vertex {
        preprocess_stage(vertex, keywords)?;
    }

    if let Some(fragment) = &mut shader.source.fragment {
        preprocess_stage(fragment, keywords)?;
    }

    Ok(())
}

fn split_blocks(source: &str) -> Vec<RawShaderCodeBlock> {
    let mut blocks = Vec::new();

    let mut cur_source = Vec::new();

    for line in source.lines() {
        if line.trim().starts_with(macro_prefix!()) {
            if !cur_source.is_empty() {
                let code_block = cur_source.join("\n");
                blocks.push(RawShaderCodeBlock::Code(code_block));
                cur_source.clear();
            }

            blocks.push(RawShaderCodeBlock::Macro(
                line.trim()
                    .strip_prefix(macro_prefix!())
                    .unwrap()
                    .to_string(),
            ));
        } else {
            cur_source.push(line.to_string());
        }
    }

    if !cur_source.is_empty() {
        let code_block = cur_source.join("\n");
        blocks.push(RawShaderCodeBlock::Code(code_block));
    }

    blocks
}

fn preprocess_stage(
    stage: &mut ShaderStage,
    keywords: &HashMap<String, u32>,
) -> Result<(), PreprocessErr> {
    // Split the source into lines. We then join the lines into either a "macro" line, or multiple "code" lines

    let mut blocks = split_blocks(&stage.source)
        .into_iter()
        .map(RawShaderCodeBlock::parse)
        .collect::<Result<Vec<_>, ParseMacroErr>>()?;

    log::debug!("Blocks: {:#?}", blocks);

    while blocks
        .iter()
        .any(|b| matches!(b, ShaderCodeBlock::Macro(_)))
    {
        let (next_macro_idx, next_macro) = blocks
            .iter()
            .enumerate()
            .find_map(|(i, b)| match b {
                ShaderCodeBlock::Macro(m) => Some((i, m)),
                ShaderCodeBlock::Code(_) => None,
            })
            .unwrap();

        match next_macro {
            Macro::If {
                keyword,
                condition,
                value,
            } => {
                let kw_val = keywords.get(keyword).unwrap_or(&0);

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
            other => return Err(PreprocessErr::UnexpectedMacro(format!("{:#?}", other))),
        }

        blocks.remove(next_macro_idx);

        log::debug!("Post-stage {:#?}", blocks);
    }

    let code = blocks
        .into_iter()
        .map(|block| match block {
            ShaderCodeBlock::Macro(_) => unreachable!("All macros should have been processed"),
            ShaderCodeBlock::Code(c) => c,
        })
        .collect::<Vec<_>>()
        .join("\n");

    stage.source = code;
    Ok(())
}

fn process_branch(
    blocks: &mut Vec<ShaderCodeBlock>,
    branch_start: usize,
    is_true: bool,
) -> Result<(), PreprocessErr> {
    let mut branch_stack = 0;
    let mut else_start = None;
    let mut branch_end = None;

    for (i, block) in blocks.iter().enumerate().skip(branch_start) {
        match block {
            ShaderCodeBlock::Macro(Macro::If { .. }) => {
                branch_stack += 1;
            }
            ShaderCodeBlock::Macro(Macro::Else) => {
                if branch_stack == 0 {
                    else_start = Some(i);
                }
            }
            ShaderCodeBlock::Macro(Macro::Endif) => {
                if branch_stack == 0 {
                    branch_end = Some(i);
                    break;
                } else {
                    branch_stack -= 1;
                }
            }
            ShaderCodeBlock::Code(_) => continue,
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

#[derive(Debug)]
enum RawShaderCodeBlock {
    Macro(String),
    Code(String),
}

impl RawShaderCodeBlock {
    fn parse(self) -> Result<ShaderCodeBlock, ParseMacroErr> {
        Ok(match self {
            RawShaderCodeBlock::Macro(macro_str) => {
                ShaderCodeBlock::Macro(Macro::from_str(&macro_str)?)
            }
            RawShaderCodeBlock::Code(code) => ShaderCodeBlock::Code(code),
        })
    }
}

#[derive(Debug)]
enum ShaderCodeBlock {
    Macro(Macro),
    Code(String),
}
