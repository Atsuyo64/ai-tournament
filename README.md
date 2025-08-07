# AI Agent Evaluator

A modular Rust crate system for evaluating AI agents via customizable tournaments, supporting sandboxed execution and flexible constraints.

## Overview

This project provides tools to benchmark and evaluate AI agents in a controlled tournament setting. It supports custom games, custom tournament strategies, and execution isolation using Linux cgroups v2.

### Key Features

- **Pluggable Tournaments**: Define your own tournament logic via the `TournamentStrategy` trait, or use built-in strategies like `SwissTournament` and `SinglePlayerTournament`.
- **Custom Games**: Any environment that implements the `Game` trait can be used.
- **Sandboxed Agent Execution**: Each agent runs in its own isolated process with:

  * Dedicated CPU cores (`taskset`)
  * Memory and CPU limits (via cgroups v2)
  * Action timeouts and total time budgets
- **Configurable Constraints**: Use the `ConstraintsBuilder` to define:

  * CPUs used per agent
  * Memory limits
  * Timeouts and think-time budgets

### Usage Summary

1. Implement the `Game` trait for your task or environment.
2. Provide AI agents as Rust crates in a specified directory.
3. Define resource constraints with the `ConstraintsBuilder`.
4. Choose or implement a `TournamentStrategy`.
5. Run the evaluator to get per-agent scores, as defined by the tournament type.

## Repository Structure

```
.
├── agent-interface  # Crate defining shared traits Game, GameFactory and Agent
├── cgroup-manager   # Crate handling Linux cgroups v2 for process isolation
├── server           # Crate containing core logic: tournament runner, strategy, constraints, etc.
└── README.md        # You're here!
```

See [`server/README.md`](server/README.md) for details on the main crate.

## Usage Example

```rust
use anyhow;
use server::{
    configuration::Configuration,
    constraints::ConstraintsBuilder,
    server::Evaluator,
    tournament_strategy::{SinglePlayerScore, SinglePlayerTournament},
};
use std::{collections::HashMap, time::Duration};

// Your custom game implementing the Game + GameFactory traits
use crate::YourGame;

fn main() -> anyhow::Result<()> {
    // Define per-agent constraints
    let constraints = ConstraintsBuilder::new()
        .with_ram_per_agent(1000) // in MB
        .with_action_timeout(Duration::from_millis(100))
        .build()?;

    // Define evaluator behaviour
    let config = Configuration::new().with_allow_uncontained(true);

    let factory = YourGame::new(); // Your game logic
    let evaluator = Evaluator::new(factory, config, constraints);

    let tournament = SinglePlayerTournament::new(10); // Run 10 games per agent
    let results: HashMap<String, SinglePlayerScore> =
        evaluator.evaluate("path_to_agents_directory", tournament)?;

    // Sort and display scores
    let mut sorted = results.iter().collect::<Vec<_>>();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (agent_name, score) in sorted {
        println!("{agent_name}: {score:?}");
    }

    Ok(())
}
```

> [!NOTE]  
> Agents must be Rust crates located in the specified directory. Each agent of each match runs as a separate, isolated process.

## Example Agent

Here’s a minimal example of an agent compatible with the evaluator system. The agent connects to the evaluator’s server via TCP, reads the game state, and responds with an action:

```rust
use std::{
    env,
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    str::{FromStr},
};

use anyhow;

use YourAgent;
use YourGame;

fn main() -> anyhow::Result<()> {
    let mut args = env::args();
    let _ = args.next(); // Skip binary name

    // Read the port number to connect to
    let port = args.next().unwrap().parse().unwrap();
    let addr = SocketAddrV4::new(Ipv4Addr::from_str("127.0.0.1")?, port);
    let mut stream = TcpStream::connect(addr)?;

    let mut agent = YourAgent::new();

    // Interaction loop
    loop {
        let mut buf = [0; 4096];
        let n = stream.read(&mut buf)?;
        let string = str::from_utf8(&buf[..n]).unwrap();

        // Parse game state, compute action, send it back
        let game_state = string.parse::<YourGame::State>().unwrap();
        let action = agent.select_action(game_state)?;
        stream.write_all(action.to_string().as_bytes())?;
    }
}
```

### Requirements

- `YourGame::State` and `YourGame::Action` must implement `FromStr` and `ToString`
- The agent must connect to the provided TCP port and handle communication over the stream
- The agent's select_action call must complete before the action timeout, or it will be forcefully terminated.
