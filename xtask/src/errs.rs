//! Error handling and utilities

use core::error::Error;
use std::fmt::Display;

use derive_more::Display;
use derive_more::Error;

#[derive(Debug, Clone, Error, Display)]
struct OneOffErr(#[error(not(source))] String);

/// A quick one-off error, given by something [`Display`]-able
pub(crate) fn err<C>(e: C) -> Box<dyn Error>
where
    C: Display + Send + Sync,
{
    Box::new(OneOffErr(e.to_string()))
}

/// Small extension trait to make adding context to an error result possible
pub(crate) trait ErrContext {
    /// The success type
    type T;

    /// Add context to the given error
    fn context<C>(self, ctx: C) -> Result<Self::T, Box<dyn Error>>
    where
        C: Display + Send + Sync + 'static;

    /// Add context to the given error by calling `ctx_fn` if `self` is not [`Ok`]
    #[expect(unused, reason = "Will be useful quite soon I think")]
    fn with_context<C, F>(self, ctx_fn: F) -> Result<Self::T, Box<dyn Error>>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

#[derive(Debug, Clone, Display, Error)]
#[display("{}: {}", _0, _1)]
struct WithContext<T>(String, #[error(source)] T);

impl<T, E> ErrContext for Result<T, E>
where
    E: Error + 'static,
{
    type T = T;

    fn context<C>(self, ctx: C) -> Result<Self::T, Box<dyn Error>>
    where
        C: Display + Send + Sync + 'static,
    {
        match self {
            Self::Ok(ok) => Ok(ok),
            Self::Err(e) => Err(Box::new(WithContext(ctx.to_string(), e))),
        }
    }

    fn with_context<C, F>(self, ctx_fn: F) -> Result<Self::T, Box<dyn Error>>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        match self {
            Self::Ok(ok) => Ok(ok),
            Self::Err(e) => Err(Box::new(WithContext(ctx_fn().to_string(), e))),
        }
    }
}
