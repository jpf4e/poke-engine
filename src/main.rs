#![allow(dead_code)]

use std::mem;
use std::mem::size_of;
use std::process::exit;
use poke_engine::choices::MOVES;
use poke_engine::instruction::{ApplyVolatileStatusInstruction, BoostInstruction, ChangeItemInstruction, ChangeSideConditionInstruction, ChangeStatusInstruction, ChangeTerrain, ChangeType, ChangeWeather, DamageInstruction, DecrementWishInstruction, DisableMoveInstruction, EnableMoveInstruction, HealInstruction, IncrementWishInstruction, Instruction, RemoveVolatileStatusInstruction, SetSubstituteHealthInstruction, SwitchInstruction};
use poke_engine::search::expectiminimax_search;
use poke_engine::state::{Move, PokemonIndex, PokemonStatus, PokemonVolatileStatus, SideReference, State};

extern crate lazy_static;


struct Test {
    a: bool,
}


fn main() {
    println!("Instruction: {}", mem::size_of::<Instruction>());
    println!("SwitchInstruction: {}", mem::size_of::<SwitchInstruction>());
    println!("ChangeItemInstruction: {}", mem::size_of::<ChangeItemInstruction>());
    println!("ChangeStatusInstruction: {}", mem::size_of::<ChangeStatusInstruction>());
    println!("DisableMoveInstruction: {}", mem::size_of::<DisableMoveInstruction>());
    println!("EnableMoveInstruction: {}", mem::size_of::<EnableMoveInstruction>());
    println!("ApplyVolatileStatusInstruction: {}", mem::size_of::<ApplyVolatileStatusInstruction>());
    println!("RemoveVolatileStatusInstruction: {}", mem::size_of::<RemoveVolatileStatusInstruction>());
    println!("HealInstruction: {}", mem::size_of::<HealInstruction>());
    println!("DamageInstruction: {}", mem::size_of::<DamageInstruction>());
    println!("BoostInstruction: {}", mem::size_of::<BoostInstruction>());
    println!("ChangeSideConditionInstruction: {}", mem::size_of::<ChangeSideConditionInstruction>());
    println!("ChangeWeather: {}", mem::size_of::<ChangeWeather>());
    println!("ChangeTerrain: {}", mem::size_of::<ChangeTerrain>());
    println!("ChangeType: {}", mem::size_of::<ChangeType>());
    println!("IncrementWishInstruction: {}", mem::size_of::<IncrementWishInstruction>());
    println!("DecrementWishInstruction: {}", mem::size_of::<DecrementWishInstruction>());
    println!("DamageInstruction: {}", mem::size_of::<DamageInstruction>());
    println!("SetSubstituteHealthInstruction: {}", mem::size_of::<SetSubstituteHealthInstruction>());

    println!("usize: {}", mem::size_of::<usize>());

    exit(1);



    let mut state: State = State::default();


    state.side_one.pokemon[PokemonIndex::P0].moves = vec![
        Move {
            id: "earthquake".to_string(),
            disabled: false,
            pp: 32,
            choice: MOVES.get("earthquake").unwrap().to_owned(),
        },
        Move {
            id: "thunderwave".to_string(),
            disabled: false,
            pp: 32,
            choice: MOVES.get("thunderwave").unwrap().to_owned(),
        },
        Move {
            id: "zapcannon".to_string(),
            disabled: false,
            pp: 32,
            choice: MOVES.get("zapcannon").unwrap().to_owned(),
        },
        Move {
            id: "airslash".to_string(),
            disabled: false,
            pp: 32,
            choice: MOVES.get("airslash").unwrap().to_owned(),
        },
    ];
    state.side_two.pokemon[PokemonIndex::P0].moves = vec![
        Move {
            id: "earthquake".to_string(),
            disabled: false,
            pp: 32,
            choice: MOVES.get("earthquake").unwrap().to_owned(),
        },
        Move {
            id: "thunderwave".to_string(),
            disabled: false,
            pp: 32,
            choice: MOVES.get("thunderwave").unwrap().to_owned(),
        },
        Move {
            id: "zapcannon".to_string(),
            disabled: false,
            pp: 32,
            choice: MOVES.get("zapcannon").unwrap().to_owned(),
        },
        Move {
            id: "airslash".to_string(),
            disabled: false,
            pp: 32,
            choice: MOVES.get("airslash").unwrap().to_owned(),
        },
    ];

    // println!("{:?}", state);
    let (side_one_options, side_two_options) = state.get_all_options();

    let result = expectiminimax_search(&mut state, 3, side_one_options, side_two_options, false);
    // println!("{:?}", state);
}
