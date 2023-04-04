use std::fmt::Display;

/// Generic error type for [`Hook`] and [`HookRegistry`]
/// 
/// Used for errors when executing the associated closure
/// or general errors in [`HookRegisty`]
#[derive(Debug)]
pub struct HookError(&'static str);
impl HookError {
    pub(crate) fn new(code: &'static str) -> Self {
        Self(code)
    }
}
impl Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
