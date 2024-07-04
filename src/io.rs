use crate::choices::{Choice, Choices, MOVES};
use crate::evaluate::evaluate;
use crate::generate_instructions::{calculate_damage_rolls, generate_instructions_from_move_pair};
use crate::instruction::{Instruction, StateInstructions};
use crate::mcts::{perform_mcts, MctsResult};
use crate::search::{expectiminimax_search, iterative_deepen_expectiminimax, pick_safest};
use crate::state::{MoveChoice, Pokemon, Side, SideReference, State};
use clap::Parser;
use std::io;
use std::io::Write;
use std::process::exit;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

struct IOData {
    state: State,
    instruction_list: Vec<Vec<Instruction>>,
    last_instructions_generated: Vec<StateInstructions>,
}

#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "")]
    state: String,

    #[clap(subcommand)]
    subcmd: Option<SubCommand>,
}

#[derive(Parser)]
enum SubCommand {
    Expectiminimax(Expectiminimax),
    IterativeDeepening(IterativeDeepening),
    MonteCarloTreeSearch(MonteCarloTreeSearch),
    CalculateDamage(CalculateDamage),
}

#[derive(Parser)]
struct Expectiminimax {
    #[clap(short, long, required = true)]
    state: String,

    #[clap(short, long, default_value_t = false)]
    ab_prune: bool,

    #[clap(short, long, default_value_t = 2)]
    depth: i8,
}

#[derive(Parser)]
struct IterativeDeepening {
    #[clap(short, long, required = true)]
    state: String,

    #[clap(short, long, default_value_t = 5000)]
    time_to_search_ms: u64,
}

#[derive(Parser)]
struct MonteCarloTreeSearch {
    #[clap(short, long, required = true)]
    state: String,

    #[clap(short, long, default_value_t = 5000)]
    time_to_search_ms: u64,
}

#[derive(Parser)]
struct CalculateDamage {
    #[clap(short, long, required = true)]
    state: String,

    #[clap(short = 'o', long, required = true)]
    side_one_move: String,

    #[clap(short = 't', long, required = true)]
    side_two_move: String,
}

impl Default for IOData {
    fn default() -> Self {
        IOData {
            state: State::default(),
            instruction_list: Vec::new(),
            last_instructions_generated: Vec::new(),
        }
    }
}

impl Side {
    fn option_to_string(&self, option: &MoveChoice) -> String {
        match option {
            MoveChoice::Move(index) => {
                return format!("{}", self.get_active_immutable().moves[*index].id).to_lowercase();
            }
            MoveChoice::Switch(index) => {
                return format!("{}", self.pokemon[*index].id).to_lowercase();
            }
            MoveChoice::None => {
                return "no move".to_string();
            }
        }
    }

    fn string_to_movechoice(&self, s: &str) -> Option<MoveChoice> {
        let s = s.to_lowercase();
        if s == "none" {
            return Some(MoveChoice::None);
        }

        let mut pkmn_iter = self.pokemon.into_iter();
        while let Some(pkmn) = pkmn_iter.next() {
            if pkmn.id.to_lowercase() == s && pkmn_iter.pokemon_index != self.active_index {
                return Some(MoveChoice::Switch(pkmn_iter.pokemon_index));
            }
        }
        let mut move_iter = self.get_active_immutable().moves.into_iter();
        while let Some(mv) = move_iter.next() {
            if format!("{:?}", mv.id).to_lowercase() == s {
                return Some(MoveChoice::Move(move_iter.pokemon_move_index));
            }
        }

        return None;
    }
}

impl Pokemon {
    fn io_print(&self) -> String {
        let moves: Vec<String> = self
            .moves
            .into_iter()
            .map(|m| format!("{:?}", m.id).to_lowercase())
            .collect();
        return format!(
            "Active: {}\nHP: {}/{}\nStatus: {:?}\nAbility: {:?}\nItem: {:?}\nBoosts: {:?}\nVolatiles: {:?}\nMoves: {:?}",
            self.id,
            self.hp,
            self.maxhp,
            self.status,
            self.ability,
            self.item,
            self.get_pkmn_boost_enum_pairs(),
            self.volatile_statuses,
            moves
        );
    }
}

fn io_get_all_options(state: &State) -> (Vec<MoveChoice>, Vec<MoveChoice>) {
    if state.team_preview {
        let mut s1_options = Vec::with_capacity(6);
        let mut s2_options = Vec::with_capacity(6);

        let mut pkmn_iter = state.side_one.pokemon.into_iter();
        while let Some(_) = pkmn_iter.next() {
            s1_options.push(MoveChoice::Switch(pkmn_iter.pokemon_index));
        }
        let mut pkmn_iter = state.side_two.pokemon.into_iter();
        while let Some(_) = pkmn_iter.next() {
            s2_options.push(MoveChoice::Switch(pkmn_iter.pokemon_index));
        }
        return (s1_options, s2_options);
    }

    let (mut s1_options, mut s2_options) = state.get_all_options();

    if state.side_one.force_trapped {
        s1_options.retain(|x| match x {
            MoveChoice::Move(_) => true,
            MoveChoice::Switch(_) => false,
            MoveChoice::None => true,
        });
    }
    if state.side_one.slow_uturn_move {
        s1_options.clear();
        state
            .side_one
            .get_active_immutable()
            .add_available_moves(&mut s1_options, &state.side_one.last_used_move);
    }

    if state.side_two.force_trapped {
        s2_options.retain(|x| match x {
            MoveChoice::Move(_) => true,
            MoveChoice::Switch(_) => false,
            MoveChoice::None => true,
        });
    }
    if state.side_two.slow_uturn_move {
        s2_options.clear();
        state
            .side_two
            .get_active_immutable()
            .add_available_moves(&mut s2_options, &state.side_two.last_used_move);
    }

    return (s1_options, s2_options);
}

fn pprint_expectiminimax_result(
    result: &Vec<f32>,
    s1_options: &Vec<MoveChoice>,
    s2_options: &Vec<MoveChoice>,
    safest_choice: &(usize, f32),
    state: &State,
) {
    let s1_len = s1_options.len();
    let s2_len = s2_options.len();

    print!("{: <12}", " ");

    for s2_move in s2_options.iter() {
        match s2_move {
            MoveChoice::Move(m) => {
                let s2_move_str = format!("{}", state.side_two.get_active_immutable().moves[*m].id);
                print!("{: >12}", s2_move_str.to_lowercase());
            }
            MoveChoice::Switch(s) => {
                let s2_move_str = format!("{}", state.side_two.pokemon[*s].id.to_lowercase());
                print!("{: >12}", s2_move_str);
            }
            MoveChoice::None => {}
        }
    }
    print!("\n");

    for i in 0..s1_len {
        let s1_move_str = s1_options[i];
        match s1_move_str {
            MoveChoice::Move(m) => {
                let move_id = state.side_one.get_active_immutable().moves[m].id;
                print!("{:<12}", move_id.to_string().to_lowercase());
            }
            MoveChoice::Switch(s) => {
                let pkmn_id = &state.side_one.pokemon[s].id;
                print!("{:<12}", pkmn_id.to_lowercase());
            }
            MoveChoice::None => {}
        }
        for j in 0..s2_len {
            let index = i * s2_len + j;
            print!("{number:>11.2} ", number = result[index]);
        }
        print!("\n");
    }
    match s1_options[safest_choice.0] {
        MoveChoice::Move(m) => {
            let move_id = state.side_one.get_active_immutable().moves[m].id;
            print!(
                "\nSafest Choice: {}, {}\n",
                move_id.to_string().to_lowercase(),
                safest_choice.1
            );
        }
        MoveChoice::Switch(s) => {
            let pkmn_id = &state.side_one.pokemon[s].id;
            print!(
                "\nSafest Choice: Switch {}, {}\n",
                pkmn_id.to_lowercase(),
                safest_choice.1
            );
        }
        MoveChoice::None => println!("No Move"),
    }
}

fn pprint_mcts_result(state: &State, result: MctsResult) {
    let s1_joined_options = result
        .s1
        .iter()
        .map(|x| {
            format!(
                "{},{:.2},{}",
                get_move_id_from_movechoice(&state.side_one, &x.move_choice),
                x.total_score,
                x.visits
            )
        })
        .collect::<Vec<String>>()
        .join("|");
    let s2_joined_options = result
        .s2
        .iter()
        .map(|x| {
            format!(
                "{},{:.2},{}",
                get_move_id_from_movechoice(&state.side_two, &x.move_choice),
                x.total_score,
                x.visits
            )
        })
        .collect::<Vec<String>>()
        .join("|");

    println!("Total Iterations: {}", result.iteration_count);
    println!("side one: {}", s1_joined_options);
    println!("side two: {}", s2_joined_options);
}

fn get_move_id_from_movechoice(side: &Side, move_choice: &MoveChoice) -> String {
    return match move_choice {
        MoveChoice::Move(index) => {
            format!("{}", side.get_active_immutable().moves[*index].id).to_lowercase()
        }
        MoveChoice::Switch(index) => format!("switch {}", side.pokemon[*index].id).to_lowercase(),
        MoveChoice::None => "No Move".to_string(),
    };
}

fn print_subcommand_result(
    result: &Vec<f32>,
    side_one_options: &Vec<MoveChoice>,
    side_two_options: &Vec<MoveChoice>,
    state: &State,
) {
    let safest = pick_safest(&result, side_one_options.len(), side_two_options.len());
    let move_choice = side_one_options[safest.0];

    let joined_side_one_options = side_one_options
        .iter()
        .map(|x| format!("{}", get_move_id_from_movechoice(&state.side_one, x)))
        .collect::<Vec<String>>()
        .join(",");
    println!("side one options: {}", joined_side_one_options);

    let joined_side_two_options = side_two_options
        .iter()
        .map(|x| format!("{}", get_move_id_from_movechoice(&state.side_two, x)))
        .collect::<Vec<String>>()
        .join(",");
    println!("side two options: {}", joined_side_two_options);

    let joined = result
        .iter()
        .map(|x| format!("{:.2}", x))
        .collect::<Vec<String>>()
        .join(",");
    println!("matrix: {}", joined);
    match move_choice {
        MoveChoice::Move(_) => {
            println!(
                "choice: {}",
                get_move_id_from_movechoice(&state.side_one, &move_choice)
            );
        }
        MoveChoice::Switch(_) => {
            println!(
                "choice: switch {}",
                get_move_id_from_movechoice(&state.side_one, &move_choice)
            );
        }
        MoveChoice::None => {
            println!("no move");
        }
    }
    println!("evaluation: {}", safest.1);
}

pub fn main() {
    let args = Cli::parse();
    let mut io_data = IOData::default();

    if args.state != "" {
        let state = State::deserialize(args.state.as_str());
        io_data.state = state;
    }

    let result;
    let mut state;
    let mut side_one_options;
    let mut side_two_options;
    match args.subcmd {
        None => {
            command_loop(io_data);
            exit(0);
        }
        Some(subcmd) => match subcmd {
            SubCommand::Expectiminimax(expectiminimax) => {
                state = State::deserialize(expectiminimax.state.as_str());
                (side_one_options, side_two_options) = io_get_all_options(&state);
                result = expectiminimax_search(
                    &mut state,
                    expectiminimax.depth,
                    side_one_options.clone(),
                    side_two_options.clone(),
                    expectiminimax.ab_prune,
                    &Arc::new(Mutex::new(true)),
                );
                print_subcommand_result(&result, &side_one_options, &side_two_options, &state);
            }
            SubCommand::IterativeDeepening(iterative_deepending) => {
                state = State::deserialize(iterative_deepending.state.as_str());
                (side_one_options, side_two_options) = io_get_all_options(&state);
                (side_one_options, side_two_options, result, _) = iterative_deepen_expectiminimax(
                    &mut state,
                    side_one_options.clone(),
                    side_two_options.clone(),
                    std::time::Duration::from_millis(iterative_deepending.time_to_search_ms),
                );
                print_subcommand_result(&result, &side_one_options, &side_two_options, &state);
            }
            SubCommand::MonteCarloTreeSearch(mcts) => {
                state = State::deserialize(mcts.state.as_str());
                (side_one_options, side_two_options) = io_get_all_options(&state);
                let result = perform_mcts(
                    &mut state,
                    side_one_options.clone(),
                    side_two_options.clone(),
                    std::time::Duration::from_millis(mcts.time_to_search_ms),
                );
                pprint_mcts_result(&state, result);
            }
            SubCommand::CalculateDamage(calculate_damage) => {
                state = State::deserialize(calculate_damage.state.as_str());
                let s1_choice = MOVES
                    .get(&Choices::from_str(calculate_damage.side_one_move.as_str()).unwrap())
                    .unwrap()
                    .to_owned();
                let s2_choice = MOVES
                    .get(&Choices::from_str(calculate_damage.side_two_move.as_str()).unwrap())
                    .unwrap()
                    .to_owned();
                calculate_damage_io(&state, s1_choice, s2_choice);
            }
        },
    }

    exit(0);
}

fn calculate_damage_io(state: &State, s1_choice: Choice, s2_choice: Choice) {
    let damages_dealt_s1 = calculate_damage_rolls(
        state.clone(),
        &SideReference::SideOne,
        s1_choice.clone(),
        &s2_choice,
    );
    let damages_dealt_s2 = calculate_damage_rolls(
        state.clone(),
        &SideReference::SideTwo,
        s2_choice,
        &s1_choice,
    );

    for dmg in [damages_dealt_s1, damages_dealt_s2] {
        match dmg {
            Some(damages_vec) => {
                let joined = damages_vec
                    .iter()
                    .map(|x| format!("{:?}", x))
                    .collect::<Vec<String>>()
                    .join(",");
                println!("Damage Rolls: {}", joined);
            }
            None => {
                println!("Damage Rolls: 0");
            }
        }
    }
}

fn command_loop(mut io_data: IOData) {
    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {}
            Err(error) => {
                println!("Error reading input: {}", error);
                continue;
            }
        }
        let mut parts = input.trim().split_whitespace();
        let command = parts.next().unwrap_or("");
        let mut args = parts;

        match command {
            "state" | "s" => {
                let state_string;
                match args.next() {
                    Some(s) => {
                        state_string = s;
                        let state = State::deserialize(state_string);
                        io_data.state = state;
                        println!("state initialized");
                    }
                    None => {
                        println!("Expected state string");
                    }
                }
                println!("{:?}", io_data.state);
            }
            "serialize" | "ser" => {
                println!("{}", io_data.state.serialize());
            }
            "matchup" | "m" => {
                let p1_active = io_data.state.side_one.get_active_immutable();
                let p2_active = io_data.state.side_two.get_active_immutable();
                let (side_one_options, side_two_options) = io_get_all_options(&io_data.state);

                let mut side_one_switch_pkmn = vec![];
                for pkmn in io_data.state.side_one.pokemon.into_iter() {
                    side_one_switch_pkmn.push(format!("{}: {}/{}", &pkmn.id, pkmn.hp, pkmn.maxhp));
                }
                let mut side_one_choices = vec![];
                for option in side_one_options {
                    side_one_choices.push(
                        format!("{:?}", io_data.state.side_one.option_to_string(&option))
                            .to_lowercase(),
                    );
                }

                let mut side_two_switch_pkmn = vec![];
                for pkmn in io_data.state.side_two.pokemon.into_iter() {
                    side_two_switch_pkmn.push(format!("{}: {}/{}", &pkmn.id, pkmn.hp, pkmn.maxhp));
                }
                let mut side_two_choices = vec![];
                for option in side_two_options {
                    side_two_choices.push(
                        format!("{:?}", io_data.state.side_two.option_to_string(&option))
                            .to_lowercase(),
                    );
                }

                println!(
                    "{}\nPokemon: {:?}\nAvailable Choices: [{}]\n\nvs\n\n{}\nPokemon: {:?}\nAvailable Choices: [{}]\n",
                    p1_active.io_print(),
                    side_one_switch_pkmn,
                    side_one_choices.join(", "),
                    p2_active.io_print(),
                    side_two_switch_pkmn,
                    side_two_choices.join(", "),
                );
            }
            "generate-instructions" | "g" => {
                let (s1_move, s2_move);
                match args.next() {
                    Some(s) => match io_data.state.side_one.string_to_movechoice(s) {
                        Some(m) => {
                            s1_move = m;
                        }
                        None => {
                            println!("Invalid move choice for side one: {}", s);
                            continue;
                        }
                    },
                    None => {
                        println!("Usage: generate-instructions <side-1 move> <side-2 move>");
                        continue;
                    }
                }
                match args.next() {
                    Some(s) => match io_data.state.side_two.string_to_movechoice(s) {
                        Some(m) => {
                            s2_move = m;
                        }
                        None => {
                            println!("Invalid move choice for side two: {}", s);
                            continue;
                        }
                    },
                    None => {
                        println!("Usage: generate-instructions <side-1 choice> <side-2 choice>");
                        continue;
                    }
                }
                let instructions =
                    generate_instructions_from_move_pair(&mut io_data.state, &s1_move, &s2_move);
                println!("{:?}", instructions);
                io_data.last_instructions_generated = instructions;
            }
            "calculate-damage" | "d" => {
                let (s1_choice, s2_choice);
                match args.next() {
                    Some(s) => {
                        s1_choice = MOVES
                            .get(&Choices::from_str(s).unwrap())
                            .unwrap()
                            .to_owned();
                    }
                    None => {
                        println!("Usage: calculate-damage <side-1 move> <side-2 move>");
                        continue;
                    }
                }
                match args.next() {
                    Some(s) => {
                        s2_choice = MOVES
                            .get(&Choices::from_str(s).unwrap())
                            .unwrap()
                            .to_owned();
                    }
                    None => {
                        println!("Usage: calculate-damage <side-1 move> <side-2 move>");
                        continue;
                    }
                }
                calculate_damage_io(&io_data.state, s1_choice, s2_choice);
            }
            "instructions" | "i" => {
                println!("{:?}", io_data.last_instructions_generated);
            }
            "evaluate" | "ev" => {
                println!("Evaluation: {}", evaluate(&io_data.state));
            }
            "iterative-deepening" | "id" => match args.next() {
                Some(s) => {
                    let max_time_ms = s.parse::<u64>().unwrap();
                    let (side_one_options, side_two_options) = io_get_all_options(&io_data.state);

                    let start_time = std::time::Instant::now();
                    let (s1_moves, s2_moves, result, depth_searched) =
                        iterative_deepen_expectiminimax(
                            &mut io_data.state,
                            side_one_options.clone(),
                            side_two_options.clone(),
                            std::time::Duration::from_millis(max_time_ms),
                        );
                    let elapsed = start_time.elapsed();

                    let safest_choice = pick_safest(&result, s1_moves.len(), s2_moves.len());

                    pprint_expectiminimax_result(
                        &result,
                        &s1_moves,
                        &s2_moves,
                        &safest_choice,
                        &io_data.state,
                    );
                    println!("Took: {:?}", elapsed);
                    println!("Depth Searched: {}", depth_searched);
                }
                None => {
                    println!("Usage: iterative-deepening <timeout_ms>");
                    continue;
                }
            },
            "monte-carlo-tree-search" | "mcts" => match args.next() {
                Some(s) => {
                    let max_time_ms = s.parse::<u64>().unwrap();
                    let (side_one_options, side_two_options) = io_get_all_options(&io_data.state);

                    let start_time = std::time::Instant::now();
                    let result = perform_mcts(
                        &mut io_data.state,
                        side_one_options.clone(),
                        side_two_options.clone(),
                        std::time::Duration::from_millis(max_time_ms),
                    );
                    let elapsed = start_time.elapsed();
                    pprint_mcts_result(&io_data.state, result);

                    println!("Took: {:?}", elapsed);
                }
                None => {
                    println!("Usage: monte-carlo-tree-search <timeout_ms>");
                    continue;
                }
            },
            "apply" | "a" => match args.next() {
                Some(s) => {
                    let index = s.parse::<usize>().unwrap();
                    let instructions = io_data.last_instructions_generated.remove(index);
                    io_data
                        .state
                        .apply_instructions(&instructions.instruction_list);
                    io_data.instruction_list.push(instructions.instruction_list);
                    io_data.last_instructions_generated = Vec::new();
                }
                None => {
                    println!("Usage: apply <instruction index>");
                    continue;
                }
            },
            "pop" | "p" => {
                if io_data.instruction_list.is_empty() {
                    println!("No instructions to pop");
                    continue;
                }
                let instructions = io_data.instruction_list.pop().unwrap();
                io_data.state.reverse_instructions(&instructions);
            }
            "pop-all" | "pa" => {
                for i in io_data.instruction_list.iter().rev() {
                    io_data.state.reverse_instructions(i);
                }
                io_data.instruction_list.clear();
            }
            "expectiminimax" | "e" => match args.next() {
                Some(s) => {
                    let mut ab_prune = false;
                    match args.next() {
                        Some(s) => ab_prune = s.parse::<bool>().unwrap(),
                        None => {}
                    }
                    let depth = s.parse::<i8>().unwrap();
                    let (side_one_options, side_two_options) = io_get_all_options(&io_data.state);
                    let start_time = std::time::Instant::now();
                    let result = expectiminimax_search(
                        &mut io_data.state,
                        depth,
                        side_one_options.clone(),
                        side_two_options.clone(),
                        ab_prune,
                        &Arc::new(Mutex::new(true)),
                    );
                    let elapsed = start_time.elapsed();

                    let safest_choice =
                        pick_safest(&result, side_one_options.len(), side_two_options.len());
                    pprint_expectiminimax_result(
                        &result,
                        &side_one_options,
                        &side_two_options,
                        &safest_choice,
                        &io_data.state,
                    );
                    println!("\nTook: {:?}", elapsed);
                }
                None => {
                    println!("Usage: expectiminimax <depth> <ab_prune=false>");
                    continue;
                }
            },
            "" => {
                continue;
            }
            "exit" | "quit" | "q" => {
                break;
            }
            command => {
                println!("Unknown command: {}", command);
            }
        }
    }
}
