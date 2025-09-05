# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [3.0.1](https://github.com/Atsuyo64/ai-tournament/compare/v2.0.0...v3.0.1) - 2025-09-05

### Added

- [**breaking**] Evaluator::evaluate returns a tuple: agent's score and compile errors
- delete content of selected log file at the beginning
- can create log directory if it does not exists
- pipe agent's stdout-stderr accordingly depending of 'log' and 'debug_stderr config'
- *(dev)* add optional log file to agent struct
- log compilation to compile.txt
- log file now in chosen directory
- log is now a directory parameter instead of just bool
- recursive swiss pairing
- better debug
- no-replay swiss with probably too many bye
- *(log)* swiss tournament logs aggregated results (info-level)
- *(debug)* can now debug cgroup on error if DEBUG_CGROUP is set

### Fixed

- clippy warnings
- *(log)* Logs now also works with all_config configuration
- should now compile on non-linux targets
- missing multiple matches per pair
- typo + debug prints
- RPS test agent not compiling

### Other

- removing untracked Cargo.lock files
- ci passing badge
- adding buildandtest workflow
- fix clippy warnings
- lowering MSRV
- updating todo

## [2.0.0](https://github.com/Atsuyo64/AI-agent-evaluator/compare/v1.0.0...v2.0.0) - 2025-08-25

### Added

- generic game score

### Other

- removing unused file/struct (confrontation.rs)
