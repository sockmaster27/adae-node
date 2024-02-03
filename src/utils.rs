use std::error::Error;

use neon::{context::Context, result::NeonResult};

/// A trait for extending the `Result` type with methods which are convenient within this specific project.
pub trait ResultExt<T> {
    fn or_throw<'a, C>(self, cx: &mut C) -> NeonResult<T>
    where
        C: Context<'a>;
}
impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Error,
{
    /// Converts a `Result` into a `NeonResult` by throwing an error if the `Result` is an `Err`,
    /// using the message supplied by `fmt::Display`.
    fn or_throw<'a, C>(self, cx: &mut C) -> NeonResult<T>
    where
        C: Context<'a>,
    {
        self.or_else(|e| cx.throw_error(format!("{e}")))
    }
}
