# Tournament Server

This crate contains the core evaluation system: scheduling tournaments, applying constraints, and launching isolated agent processes.

## Highlights

- Supports any user-defined `Game`
- Supports any user-defined tournament logic via the `TournamentStrategy` trait
- Comes with `SwissTournament` and `RoundRobin` implementations
- Agent processes are sandboxed with:
  * Independent CPU affinity
  * Memory and runtime limits
- Returns score with format defined by the tournament strategy used

## How It Works

- Each match consists of one or more agents, each running as a separate OS process with isolated constraints
- Agents are provided as Rust crates in a directory
- Matches are coordinated based on the selected tournament strategy

## Notes

- Requires Linux with **cgroups v2** and `taskset` installed
- Current process isolation features are Linux-only

#### License

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.


Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
