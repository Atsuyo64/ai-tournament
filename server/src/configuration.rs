//! Config for the evaluator behaviors

/// Configuration for evaluator behaviors.
#[derive(Debug, Clone, Copy)]
pub struct Configuration {
    pub(crate) verbose: bool,
    pub(crate) log: bool,
    pub(crate) allow_uncontained: bool,
    pub(crate) compile_agents: bool,
    pub(crate) self_test: bool,
    pub(crate) test_all_configs: bool,
}

impl Configuration {
    /// Create a new configuration with default parameters.
    ///
    /// By default:
    /// - The evaluator will print match progress to stdout.
    /// - Logging to file is disabled.
    /// - Unsafe fallbacks (e.g. skipping taskset or cgroup checks) are not allowed.
    /// - Agents will be compiled before execution.
    /// - Self-test mode is disabled (expects multiple agents).
    /// - Only the 'eval' configuration will be tested.
    pub fn new() -> Self {
        Self {
            verbose: true,
            log: false,
            allow_uncontained: false,
            compile_agents: true,
            self_test: false,
            test_all_configs: false,
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

    /// Enable or disable agent compilation.
    pub fn with_compile_agents(mut self, value: bool) -> Self {
        self.compile_agents = value;
        self
    }

    /// Enable or disable self-test mode.
    ///
    /// When enabled, evaluates a single agent in the current directory.
    /// When disabled, expects multiple agents in the given directory.
    pub fn with_self_test(mut self, value: bool) -> Self {
        self.self_test = value;
        self
    }

    /// Enable or disable testing all configurations.
    ///
    /// When enabled, tests every available configuration.
    /// When disabled, only tests the default `eval` configuration. (see crate documentation for
    /// more informations)
    pub fn with_test_all_configs(mut self, value: bool) -> Self {
        self.test_all_configs = value;
        self
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}
