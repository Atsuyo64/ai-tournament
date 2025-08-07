//! Config for the evaluator behaviors

/// Struct containing configuration fo evaluator behaviors
#[derive(Debug, Clone, Copy)]
pub struct Configuration {
    pub(crate) silent: bool,
    pub(crate) log: bool,
    pub(crate) allow_unsafe: bool,
}

impl Configuration {
    /// Create new Config with default parameters
    ///
    /// By default, the evaluator will pretty-print running matches, will not create a log file,
    /// and will panic if cgroups v2 or taskset is missing.
    pub fn new() -> Self {
        Self {
            silent: false,
            log: false,
            allow_unsafe: false,
        }
    }

    /// Create new Config from environment variables
    pub fn from_env() -> Self {
        //TODO:
        todo!("Config::from_env")
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}
