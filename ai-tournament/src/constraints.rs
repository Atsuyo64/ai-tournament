//! Defines resource constraints for AI agent execution.
//!
//! This module provides tools to configure and enforce per-agent and global resource limits
//! during tournament evaluation. Constraints include memory usage, CPU allocation, and timing
//! restrictions, and are applied using Linux cgroups v2 and `taskset`.
//!
//! # Overview
//!
//! The main entry point is the [`ConstraintsBuilder`] struct, which uses a builder pattern
//! to configure limits. These include:
//!
//! - **Memory constraints**: max total RAM and per-agent RAM limits
//! - **CPU constraints**: total CPU count, CPU affinity via list/range, CPUs per agent
//! - **Timing constraints**:
//!   * Per-action timeout
//!   * Total think time ("time budget") per agent across a match
//!
//! Once built, a [`Constraints`] object can be passed to the evaluator to enforce limits
//! at runtime. Internally, constraints are enforced Linux cgroups v2 and taskset.
//!
//! # Linux-Only
//!
//! This module **only works on Linux** systems with **cgroups v2 enabled**. Attempts to run
//! on other platforms will constrain only timing.
//!
//! # Example
//!
//! ```no_run
//! use std::time::Duration;
//! use ai_tournament::constraints::ConstraintsBuilder;
//!
//! let constraints = ConstraintsBuilder::new()
//!     .with_max_total_ram(16_000)
//!     .with_ram_per_agent(2_000)
//!     .with_cpu_list("0-3")
//!     .with_cpus_per_agent(2)
//!     .with_time_budget(Duration::from_secs(600))
//!     .with_action_timeout(Duration::from_millis(200))
//!     .build()
//!     .unwrap();
//! ```
//!
//! You may also construct constraints from environment variables using
//! [`ConstraintsBuilder::from_env()`] for runtime configurability.

use std::{collections::HashSet, env, time::Duration};

use anyhow::{bail, Context};
use tracing::warn;

#[derive(Debug, Default)]
enum AutoCpus {
    #[default]
    Auto,
    Count(usize),
    List(String),
}

/// A builder for defining resource constraints for agent execution environments.
///
/// This builder is used to configure limits on memory, CPU usage, and execution time
/// for agents being evaluated. It supports both
/// per-agent and global constraints, and is designed to be chainable.
///
/// By default, all constraints are unlimited, except for the total CPU count,
/// which defaults to the number of physical CPUs on the host machine, and with
/// one CPU per agent.
///
/// # Examples
///
/// ```
/// # use std::time::Duration;
/// # use ai_tournament::constraints::ConstraintsBuilder;
///
/// let constraints = ConstraintsBuilder::new()
///     .with_max_total_ram(16_000)
///     .with_ram_per_agent(2_000)
///     .with_cpu_list("0-3,6")
///     .with_cpus_per_agent(2)
///     .with_time_budget(Duration::from_secs(1200))
///     .with_action_timeout(Duration::from_millis(500))
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct ConstraintsBuilder {
    total_ram: Option<usize>,
    agent_ram: Option<usize>,
    cpus: AutoCpus,
    cpus_per_agent: Option<usize>,
    time_budget: Option<Duration>,
    action_time: Option<Duration>,
}

impl ConstraintsBuilder {
    /// Creates a new `ConstraintsBuilder` with no limits except for total CPU count,
    /// which defaults to the number of physical CPUs on the host machine, and one CPU per agent.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `ConstraintsBuilder` configured from environment variables,
    /// with no limits except for total CPU count defaulting to the number of physical CPUs and one CPU per agent.
    ///
    /// Read environment variables are:
    /// - `MAX_TOTAL_RAM` (usize): maximum total RAM in bytes
    /// - `RAM_PER_AGENT` (usize): maximum RAM per agent in bytes
    /// - `CPU_LIST` (string): comma-separated list or ranges of CPUs, e.g. "0-3,6"
    /// - `TOTAL_CPU_COUNT` (usize): total number of CPUs allowed, overridden by `CPU_LIST`
    /// - `CPUS_PER_AGENT` (usize): number of CPUs allowed per agent
    /// - `TIME_BUDGET_SECS` (u64): total time budget per agent in seconds
    /// - `ACTION_TIMEOUT_MS` (u64): timeout per action in milliseconds
    #[must_use]
    pub fn from_env() -> Self {
        fn parse_usize(var: &str) -> Option<usize> {
            env::var(var).ok()?.parse().ok()
        }

        fn parse_duration_secs(var: &str) -> Option<Duration> {
            env::var(var)
                .ok()?
                .parse::<u64>()
                .ok()
                .map(Duration::from_secs)
        }

        fn parse_duration_millis(var: &str) -> Option<Duration> {
            env::var(var)
                .ok()?
                .parse::<u64>()
                .ok()
                .map(Duration::from_millis)
        }

        let max_total_ram = parse_usize("MAX_TOTAL_RAM");
        let ram_per_agent = parse_usize("RAM_PER_AGENT");
        let cpu_list = env::var("CPU_LIST").ok();
        let total_cpu_count = parse_usize("TOTAL_CPU_COUNT");
        let cpus_per_agent = parse_usize("CPUS_PER_AGENT");
        let time_budget = parse_duration_secs("TIME_BUDGET_SECS");
        let action_timeout = parse_duration_millis("ACTION_TIMEOUT_MS");

        let cpus = if let Some(cpus_str) = cpu_list {
            AutoCpus::List(cpus_str)
        } else if let Some(count) = total_cpu_count {
            AutoCpus::Count(count)
        } else {
            AutoCpus::Auto
        };

        ConstraintsBuilder {
            total_ram: max_total_ram,
            agent_ram: ram_per_agent,
            cpus,
            cpus_per_agent,
            time_budget,
            action_time: action_timeout,
        }
    }

    /// Sets the maximum total RAM available across all agents (in MB).
    #[must_use]
    pub fn with_max_total_ram(self, max: usize) -> Self {
        Self {
            total_ram: Some(max),
            ..self
        }
    }

    /// Sets the maximum RAM available per agent (in MB).
    #[must_use]
    pub fn with_ram_per_agent(self, max: usize) -> Self {
        Self {
            agent_ram: Some(max),
            ..self
        }
    }

    /// Sets the specific CPUs available for agents using a CPU list string.
    ///
    /// Format follows the pattern: `"0-3,6,8"` (inclusive ranges and individual IDs).
    #[must_use]
    pub fn with_cpu_list(self, cpus: &str) -> Self {
        Self {
            cpus: AutoCpus::List(cpus.to_string()),
            ..self
        }
    }

    /// Sets the total number of logical CPUs available across all agents.
    ///
    /// This will be ignored if `with_cpu_list` is also specified.
    #[must_use]
    pub fn with_total_cpu_count(self, max: usize) -> Self {
        if let AutoCpus::List(_) = self.cpus {
            warn!("`with_total_cpu_count` is ignored if `with_cpu_list` is used!");
            self
        } else {
            Self {
                cpus: AutoCpus::Count(max),
                ..self
            }
        }
    }

    /// Sets the number of logical CPUs available per agent.
    ///
    /// Default is one
    #[must_use]
    pub fn with_cpus_per_agent(self, max: usize) -> Self {
        Self {
            cpus_per_agent: Some(max),
            ..self
        }
    }

    //NOTE: alt names: cumulative timeout, cumulative time limit, per agent time limit, total time limit
    /// Sets the total allowed clock-time for an agent across the entire game.
    ///
    /// This acts as a time budget. If exceeded, the agent is considered out of time.
    #[must_use]
    pub fn with_time_budget(self, duration: Duration) -> Self {
        Self {
            time_budget: Some(duration),
            ..self
        }
    }

    /// Sets the maximum duration allowed for a single action or decision made by an agent.
    ///
    /// Used to limit "thinking time" per move or step.
    #[must_use]
    pub fn with_action_timeout(self, duration: Duration) -> Self {
        Self {
            action_time: Some(duration),
            ..self
        }
    }

    /// Consumes the builder and returns the constructed `Constraints`.
    ///
    /// # Returns
    ///
    /// A `Constraints` object containing all the configured limits.
    ///
    /// # Errors
    ///
    /// Returns Error (String) when Constraints are impossible, e.g. total RAM < agent RAM
    pub fn build(self) -> anyhow::Result<Constraints> {
        let mut sys = sysinfo::System::new();

        let total_ram = self.total_ram.map(|i| i * 1_000_000).unwrap_or_else(|| {
            sys.refresh_memory();
            //REVIEW: sys.total_memory() ?
            sys.available_memory() as usize
        });

        if total_ram < (self.agent_ram.unwrap_or(0) * 1_000_000) {
            bail!(
                "Agent RAM size ({}MB) is greater than total RAM ({}MB)",
                self.agent_ram.unwrap_or(0),
                total_ram / 1_000_000
            );
        }

        // By default, we use the physical CPU count because using all logical CPUs
        // cuts agent performance in half (at least on the machine I tested).
        let cpus = match self.cpus {
            AutoCpus::Auto => {
                sys.refresh_cpu_all();
                let num_cpus = num_cpus::get_physical() as u8;
                (0..num_cpus).collect::<HashSet<u8>>()
            }
            AutoCpus::Count(num_cpus) => (0..(num_cpus as u8)).collect::<HashSet<u8>>(),
            AutoCpus::List(s) => {
                cpu_list_to_hashset(&s).map_err(|e| e.context("error parsing cpu list"))?
            }
        };
        let cpus_per_agent = self.cpus_per_agent.unwrap_or(1);
        let agent_ram = self
            .agent_ram
            .map(|i| i * 1_000_000)
            .unwrap_or_else(|| total_ram / (cpus.len() / cpus_per_agent));
        let time_budget = self.time_budget.unwrap_or(Duration::MAX);
        let action_time = self.action_time.unwrap_or(Duration::MAX);

        Ok(Constraints {
            total_ram,
            agent_ram,
            cpus,
            cpus_per_agent,
            time_budget,
            action_time,
        })
    }
}

fn cpu_list_to_hashset(s: &str) -> anyhow::Result<HashSet<u8>> {
    if s.is_empty() {
        bail!("Empty string");
    }
    let mut set: HashSet<u8> = HashSet::new();
    for item in s.split(',') {
        let mut split = item.split('-');
        let cnt = item.split('-').count();
        if cnt == 1 {
            let value: &str = split.next().unwrap();
            let value: u8 = value
                .parse()
                .with_context(|| format!("could not parse {value}"))?;
            set.insert(value);
        } else if cnt == 2 {
            let start: &str = split.next().unwrap();
            let start: u8 = start
                .parse()
                .with_context(|| format!("could not parse {start}"))?;
            let end: &str = split.next().unwrap();
            let end: u8 = end
                .parse()
                .with_context(|| format!("could not parse {end}"))?;
            let range = if start <= end {
                start..=end
            } else {
                end..=start
            };
            for i in range {
                set.insert(i);
            }
        } else {
            bail!(
                "each comma-separated item must be a number or a range (e.g. '0-3'), got '{item}'"
            );
        }
    }
    Ok(set)
}

/// Obtained using `ConstraintsBuilder`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constraints {
    pub(crate) total_ram: usize,
    pub(crate) agent_ram: usize,
    pub(crate) cpus: HashSet<u8>,
    pub(crate) cpus_per_agent: usize,
    pub(crate) time_budget: Duration,
    pub(crate) action_time: Duration,
}

impl Constraints {
    /// create a ConstraintsBuilder
    pub fn builder() -> ConstraintsBuilder {
        ConstraintsBuilder::new()
    }

    pub(crate) fn add(&mut self, res: Constraints) {
        self.total_ram += res.total_ram;
        self.cpus.extend(res.cpus);
    }

    pub(crate) fn take(&mut self, num_cpus: usize, ram: usize) -> Constraints {
        let mut cpus = HashSet::new();
        for _ in 0..num_cpus {
            cpus.insert(self.take_one_cpu());
        }
        self.total_ram -= ram;
        Constraints {
            total_ram: ram,
            cpus,
            ..*self
        }
    }

    pub(crate) fn try_take(&mut self, num_cpus: usize, ram: usize) -> Option<Constraints> {
        if self.cpus.len() >= num_cpus && self.total_ram >= ram {
            Some(self.take(num_cpus, ram))
        } else {
            None
        }
    }

    pub(crate) fn take_one_cpu(&mut self) -> u8 {
        let cpu = *self.cpus.iter().next().unwrap();
        self.cpus.take(&cpu).unwrap()
    }
}
