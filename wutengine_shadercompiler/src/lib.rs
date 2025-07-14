//! WutEngine Shader Compiler library

use core::ops::{Range, RangeInclusive};
use std::collections::HashMap;

use rayon::prelude::*;
use thiserror::Error;

use crate::ir::{PreprocessErr, preprocess};

mod branch;
mod ir;

#[derive(Debug, Clone)]
pub enum KeywordValue {
    Single(i64),
    Range(RangeInclusive<i64>),
}

impl KeywordValue {
    fn flatten<K>(&self, key: K) -> Vec<HashMap<K, i64>>
    where
        K: core::hash::Hash + Eq + Clone,
    {
        let vals = match self {
            KeywordValue::Single(single) => vec![*single],
            KeywordValue::Range(range) => range.clone().map(|i| i).collect(),
        };

        vals.into_iter()
            .map(|flat_val| {
                let mut hmap = HashMap::with_capacity(1);
                hmap.insert(key.clone(), flat_val);
                hmap
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ShaderOutput<'a> {
    pub source: String,
    pub keyword_hash: u128,
    pub keywords: HashMap<&'a str, i64>,
}

pub fn gather_compile_jobs<'a>(
    keywords: &HashMap<&'a str, KeywordValue>,
) -> Vec<HashMap<&'a str, i64>> {
    let mut jobs = Vec::new();

    for (&keyword, value) in keywords {
        if jobs.is_empty() {
            jobs.extend(value.flatten(keyword));
        } else {
            let old_jobs = std::mem::take(&mut jobs);

            for job in old_jobs {
                for new_keyword_map in value.flatten(keyword) {
                    let mut new_job = job.clone();
                    new_job.extend(new_keyword_map);
                    jobs.push(new_job);
                }
            }
        }
    }

    jobs
}

pub fn compile<'a>(
    raw_shader: &str,
    keywords: &HashMap<&'a str, KeywordValue>,
) -> Result<Vec<ShaderOutput<'a>>, Error> {
    log::info!("Compiling shader with {} keywords", keywords.len());

    let jobs = gather_compile_jobs(keywords);

    log::info!("Running {} total compile jobs", jobs.len());

    jobs.into_par_iter()
        .map(|job_keywords| compile_single(raw_shader, job_keywords))
        .collect()
}

pub fn compile_single<'a>(
    raw_shader: &str,
    keywords: HashMap<&'a str, i64>,
) -> Result<ShaderOutput<'a>, Error> {
    Ok(ShaderOutput {
        source: preprocess(raw_shader, &keywords)?,
        keyword_hash: wutengine_util::keyword_hash(&keywords),
        keywords,
    })
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error during preprocessing: {0}")]
    Preprocess(#[from] PreprocessErr),
}
