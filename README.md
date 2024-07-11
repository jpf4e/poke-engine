# Poke Engine

An engine for simulating and searching through Pokémon battles (singles only).

**This is not a perfect engine**

This battle engine is meant to capture important aspects of Pokémon for the purposes of competitive single battles.
It is nowhere near as complete or robust as the [PokemonShowdown](https://github.com/smogon/pokemon-showdown) battle engine.

### Building & Running

[Features](https://doc.rust-lang.org/cargo/reference/features.html) are used to conditionally compile code for different generations of Pokemon.
The simplest way to build the project is with the Makefile.

e.g. To build for generation 4:

```shell
make gen4
```

Run with
    
```shell
./target/release/poke-engine
```

Generations 4 through 8 are available

### Usage

There are several ways to interact with the engine through subcommands:

1. **Expectiminimax**
```shell
poke-engine expectiminimax --state <state-string> --depth <depth> [--ab-prune]
```
Search through the state using [expectiminimax](https://en.wikipedia.org/wiki/Expectiminimax) to the given depth.
Displays the results along with the best move found.

2. **Iterative Deepening**
```shell
poke-engine iterative-deepening --state <state-string> --time-to-search-ms <time>
```
Similar to expectiminimax, search through the state but use iterative deepening.
Searches for the given amount of time, then returns the best move found.

3. **Monte Carlo Tree Search**
```shell
poke-engine monte-carlo-tree-search --state <state-string> --time-to-search-ms <time>
```
Search through the state using [Monte Carlo Tree Search](https://en.wikipedia.org/wiki/Monte_Carlo_tree_search) for the given amount of time.

4. **Calculate Damage**
```shell
poke-engine calculate-damage --state <state-string> -o <s1_move> -t <s2_move>
```
Calculate the damage rolls for the given moves.

5. **Interactive Mode**: Run the engine and input commands directly
```shell
poke-engine --state <state-string>
```

Available commands:

| Command                                               | Shorthand | Function                                                                                                      |
|-------------------------------------------------------|:---------:|---------------------------------------------------------------------------------------------------------------|
| **state** *state-string*                              |     s     | Reset the state to *state-string*                                                                             |
| **matchup**                                           |     m     | Display some information about the current state                                                              |
| **generate-instructions** *side-1-move* *side-2-move* |     g     | Generate all of the instructions that would be applied to the state if side 1 and side 2 used the given moves |
| **instructions**                                      |     i     | Display the last instructions generated by **generate-instructions**                                          |
| **apply** *instruction-index*                         |     a     | Apply the last instructions instructions to the state, modifying it                                           |
| **pop**                                               |     p     | Pops the last instructions from the state, undoing their changes                                              |
| **pop-all**                                           |    pa     | Pops all applied instructions from the state                                                                  |
| **evaluate**                                          |    ev     | Calculate the current state's evaluation                                                                      |
| **calculate-damage** *side-1-move* *side-2-move*      |     d     | Calculate the damage rolls for the given moves                                                                |
| **expectiminimax** *depth* *[ab-prune=false]*         |     e     | Perform expectiminimax (see above), and display the results                                                   |
| **iterative-deepening** *time-ms*                     |    id     | Perform iterative-deepening (see above), and display the results                                              |
| **monte-carlo-tree-search** *time-ms*                 |   mcts    | Perform monte-carlo-tree-search (see above), and display the results                                          |
| **serialize**                                         |    ser    | Display the current state's serialized string                                                                 |
| **exit/quit**                                         |     q     | Quit interactive mode                                                                                         |


### State Representation

The engine parses the state of the game from a string.

Properly representing the state of a Pokémon battle gets really complicated.
See the doctest for `State::deserialize` in [serialize.rs](src/serialize.rs)
for the source of truth on how to parse a state string.
