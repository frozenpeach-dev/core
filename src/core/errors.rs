use std::fmt::Display;


#[derive(Debug)]
pub struct HookError(usize);
impl HookError {
    pub(crate) fn new(code: i32) -> Self {
        Self(code as usize)
    }
}
impl Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
