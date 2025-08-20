//! Config for the evaluator behaviors
//!
//! This module provides configuration options for controlling the behavior of the evaluator.
//!
//! Configuration can be created programmatically using [`Configuration::new()`] or by reading
//! environment variables using [`Configuration::from_env()`].
//!
//! # Environment Variables
//!
//! The following environment variables can be used to override configuration values. All
//! values are optional, and case-insensitive. Set the value to `"true"` to enable a flag.
//!
//! - `EVAL_VERBOSE` — Enable verbose output (default: `true`)
//! - `EVAL_LOG` — Enable logging to a file (default: `false`)
//! - `EVAL_ALLOW_UNCONTAINED` — Allow unsafe fallbacks (e.g., skipping `taskset`, `cgroup`) (default: `false`)
//! - `EVAL_COMPILE_AGENTS` — Compile agents before evaluation (default: `true`)
//! - `EVAL_SELF_TEST` — Enable self-test mode (for single-agent evaluation) (default: `false`)
//! - `EVAL_TEST_ALL_CONFIGS` — Test all available configurations instead of just `eval` (default: `false`)
//! - `EVAL_DEBUG_AGENT_STDERR` — Print agent stderr for debugging (default: `false`)

/// Configuration for evaluator behaviors.
#[derive(Debug, Clone, Copy)]
pub struct Configuration {
    pub(crate) verbose: bool,
    pub(crate) log: bool,
    pub(crate) allow_uncontained: bool,
    pub(crate) compile_agents: bool,
    pub(crate) self_test: bool,
    pub(crate) test_all_configs: bool,
    pub(crate) debug_agent_stderr: bool,
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
    /// - Agent stderr output is disabled
    pub fn new() -> Self {
        Self {
            verbose: true,
            log: false,
            allow_uncontained: false,
            compile_agents: true,
            self_test: false,
            test_all_configs: false,
            debug_agent_stderr: false, // default value
        }
    }

    /// Create configuration from environment variables.
    ///
    /// The following environment variables are recognized:
    /// - `EVAL_VERBOSE`: if set to `"true"`, enables verbose output (default: `true`)
    /// - `EVAL_LOG`: if set to `"true"`, enables logging to file (default: `false`)
    /// - `EVAL_ALLOW_UNCONTAINED`: if set to `"true"`, allows unsafe fallbacks (default: `false`)
    /// - `EVAL_COMPILE_AGENTS`: if set to `"true"`, enables agent compilation (default: `true`)
    /// - `EVAL_SELF_TEST`: if set to `"true"`, enables self-test mode (default: `false`)
    /// - `EVAL_TEST_ALL_CONFIGS`: if set to `"true"`, enables testing all configurations (default: `false`)
    /// - `EVAL_DEBUG_AGENT_STDERR`: if set to `"true"`, enables agent stderr debug output (default: `false`)
    ///
    /// Any other value (including unset) will result in using the default value for each field.
    pub fn from_env() -> Self {
        fn get_env_flag(var: &str, default: bool) -> bool {
            match std::env::var(var) {
                Ok(val) => val.eq_ignore_ascii_case("true"),
                Err(_) => default,
            }
        }

        Self {
            verbose: get_env_flag("EVAL_VERBOSE", true),
            log: get_env_flag("EVAL_LOG", false),
            allow_uncontained: get_env_flag("EVAL_ALLOW_UNCONTAINED", false),
            compile_agents: get_env_flag("EVAL_COMPILE_AGENTS", true),
            self_test: get_env_flag("EVAL_SELF_TEST", false),
            test_all_configs: get_env_flag("EVAL_TEST_ALL_CONFIGS", false),
            debug_agent_stderr: get_env_flag("EVAL_DEBUG_AGENT_STDERR", false),
        }
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
    /// When enabled, evaluates a single agent in the CURRENT directory.
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

    /// Enable or disable agent stderr output (debug purposes only).
    pub fn with_debug_agent_stderr(mut self, value: bool) -> Self {
        self.debug_agent_stderr = value;
        self
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}
