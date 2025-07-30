# Server

(I should really find an other name...)

This crate provides a way to easily create and run a tournament to evaluate AIs. The main features are:
<!-- - tested AI are independent from the  -->
- Can work with any game (or anything that implements the [Game](../agent-interface/src/lib.rs) trait)
- Ability to restrain AI (time budget, time per action, number of CPUs, maximum RAM amount, ...)
- Ability to use any kind of tournament as long as they implement the [TournamentStrategy](./src/tournament_strategy.rs) trait
- Basic tournaments are provided (Swiss tournament and Round-Robin)
- AI under test do not share CPUs
- Final scores struct is defined by the tournament
