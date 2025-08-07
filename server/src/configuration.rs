//! Config for the evaluator behaviors

/// Configuration for evaluator behaviors.
#[derive(Debug, Clone, Copy)]
pub struct Configuration {
    pub(crate) verbose: bool,
    pub(crate) log: bool,
    pub(crate) allow_uncontained: bool,
}

impl Configuration {
    /// Create a new configuration with default parameters.
    ///
    /// By default:
    /// - The evaluator will print match progress to stdout.
    /// - Logging to file is disabled.
    /// - Unsafe fallbacks (e.g. skipping taskset or cgroup checks) are not allowed.
    pub fn new() -> Self {
        Self {
            verbose: true,
            log: false,
            allow_uncontained: false,
        }
    }

    /// Create configuration from environment variables.
    pub fn from_env() -> Self {
        // TODO: implement reading configuration from environment
        todo!("Configuration::from_env")
    }

    /// Enable or disable silent mode.
    pub fn with_verbose(mut self, value: bool) -> Self {
        self.verbose = value;
        self
    }

    /// Enable or disable logging to file.
    pub fn with_log(mut self, value: bool) -> Self {
        self.log = value;
        self
    }

    /// Enable or disable unsafe fallbacks.
    pub fn with_allow_uncontained(mut self, value: bool) -> Self {
        self.allow_uncontained = value;
        self
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}
