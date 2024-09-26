mod bugcheck_codegen;
use bugcheck_codegen::BUGCHECK_CODES;

#[derive(Clone)]
pub struct BugCheckCache {}

impl BugCheckCache {
    /// Construct a new cache
    pub fn new() -> Self {
        Self {}
    }

    #[tracing::instrument(name = "bugcheck_lookup", skip(self))]
    pub fn get(&self, code: u64) -> Option<&(&str, &str)> {
        BUGCHECK_CODES.get(&code)
    }
}

impl Default for BugCheckCache {
    fn default() -> Self {
        Self::new()
    }
}
