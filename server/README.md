# Tournament Server

This crate powers the evaluation system by orchestrating tournaments, applying constraints, and launching isolated agent processes.

## Highlights

- Supports any user-defined `Game`
- Pluggable tournament logic via the `TournamentStrategy` trait
- Comes with `SwissTournament` and `RoundRobin` implementations
- Agent processes are sandboxed with:

  * Independent CPU affinity
  * Memory and runtime limits
- Score output format is defined by the tournament strategy used

## How It Works

- Agents are provided as Rust crates in a directory
- Each agent match runs in a separate process with defined constraints
- Matches are coordinated based on the selected tournament strategy
- Final results reflect the scoring logic of that strategy

## Notes

- Requires Linux with **cgroups v2** and `taskset` installed
- Current process isolation features are Linux-only