#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use data::moves::{self, Choice, Flags, MoveCategory};

use crate::{data::{moves::{Secondary, MoveTarget, Effect}, conditions::{PokemonSideCondition, PokemonStatus, PokemonVolatileStatus}}, state::{PokemonTypes, PokemonNatures, Pokemon, SideReference}};
extern crate lazy_static;

mod data;
mod state;
mod instruction;

fn main() {
    let mut _sample_choice: moves::Choice = Choice{
        id: "tackle".to_string(),
        accuracy: 100 as f32,
        category: MoveCategory::Physical,
        base_power: 40 as f32,
        boost: None,
        priority: 0,
        flags: Flags {
            authentic: false,
            bite: false,
            bullet: false,
            charge: false,
            contact: false,
            dance: false,
            defrost: false,
            distance: false,
            drag: false,
            gravity: false,
            heal: false,
            mirror: false,
            mystery: false,
            nonsky: false,
            powder: false,
            protect: false,
            pulse: false,
            punch: false,
            recharge: false,
            reflectable: false,
            snatch: false,
            sound: false,
        },
        heal: None,
        status: None,
        volatile_status: None,
        side_condition: None,
        secondaries: Some(vec![
            Secondary {
                chance: 50,
                target: MoveTarget::Opponent,
                effect: Effect::Status(PokemonStatus::Burn)
            }
        ]),
        target: MoveTarget::Opponent
    };

    let pikachu: state::Pokemon = Pokemon {
        id: "pikachu".to_string(),
        level: 100,
        types: (PokemonTypes::Electric, PokemonTypes::Typeless),
        hp: 100,
        maxhp: 100,
        ability: "voltabsorb".to_string(),
        item: "none".to_string(),
        attack: 100,
        defense: 100,
        special_attack: 100,
        special_defense: 100,
        speed: 100,
        attack_boost: 0,
        defense_boost: 0,
        special_attack_boost: 0,
        special_defense_boost: 0,
        speed_boost: 0,
        accuracy_boost: 0,
        evasion_boost: 0,
        status: PokemonStatus::None,
        nature: PokemonNatures::Serious,
        volatile_statuses: HashSet::<PokemonVolatileStatus>::new(),
        moves: vec![],
    };
    let landorustherian: state::Pokemon = Pokemon {
        id: "landorustherian".to_string(),
        level: 100,
        types: (PokemonTypes::Flying, PokemonTypes::Ground),
        hp: 100,
        maxhp: 100,
        ability: "voltabsorb".to_string(),
        item: "none".to_string(),
        attack: 100,
        defense: 100,
        special_attack: 100,
        special_defense: 100,
        speed: 100,
        attack_boost: 0,
        defense_boost: 0,
        special_attack_boost: 0,
        special_defense_boost: 0,
        speed_boost: 0,
        accuracy_boost: 0,
        evasion_boost: 0,
        status: PokemonStatus::None,
        nature: PokemonNatures::Serious,
        volatile_statuses: HashSet::<PokemonVolatileStatus>::new(),
        moves: vec![],
    };

    let my_side: state::Side = state::Side {
        active_index: 0,
        reserve: [pikachu],
        side_conditions: HashMap::<PokemonSideCondition, i8>::new(),
        wish: (0, 0),
    };

    let your_side: state::Side = state::Side {
        active_index: 0,
        reserve: [landorustherian],
        side_conditions: HashMap::<PokemonSideCondition, i8>::new(),
        wish: (0, 0),
    };

    let state_weather = state::StateWeather {
        weather_type: state::Weather::None,
        turns_remaining: 0,
    };

    let state_terrain = state::StateTerrain {
        terrain_type: state::Terrain::None,
        turns_remaining: 0,
    };

    let mut state: state::State = state::State {
        side_one: my_side,
        side_two: your_side,
        weather: state_weather,
        terrain: state_terrain,
        trick_room: false,
    };


    let pkmn = state.side_one.get_active();
    pkmn.hp -= 1;
    println!("{:?}", state.side_one.get_active().hp);

    state.damage(SideReference::SideOne, 1);

    println!("{:?}", state.side_one.get_active().hp);
}
