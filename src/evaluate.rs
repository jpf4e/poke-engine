use crate::abilities::Abilities;
use crate::choices::MoveCategory;
use crate::state::{Pokemon, PokemonStatus, PokemonVolatileStatus, State};

const POKEMON_ALIVE: f32 = 75.0;
const POKEMON_HP: f32 = 100.0;

const POKEMON_ATTACK_BOOST: f32 = 15.0;
const POKEMON_DEFENSE_BOOST: f32 = 15.0;
const POKEMON_SPECIAL_ATTACK_BOOST: f32 = 15.0;
const POKEMON_SPECIAL_DEFENSE_BOOST: f32 = 15.0;
const POKEMON_SPEED_BOOST: f32 = 25.0;

const POKEMON_BOOST_MULTIPLIER_6: f32 = 3.3;
const POKEMON_BOOST_MULTIPLIER_5: f32 = 3.15;
const POKEMON_BOOST_MULTIPLIER_4: f32 = 3.0;
const POKEMON_BOOST_MULTIPLIER_3: f32 = 2.5;
const POKEMON_BOOST_MULTIPLIER_2: f32 = 2.0;
const POKEMON_BOOST_MULTIPLIER_1: f32 = 1.0;
const POKEMON_BOOST_MULTIPLIER_0: f32 = 0.0;
const POKEMON_BOOST_MULTIPLIER_NEG_1: f32 = -1.0;
const POKEMON_BOOST_MULTIPLIER_NEG_2: f32 = -2.0;
const POKEMON_BOOST_MULTIPLIER_NEG_3: f32 = -2.5;
const POKEMON_BOOST_MULTIPLIER_NEG_4: f32 = -3.0;
const POKEMON_BOOST_MULTIPLIER_NEG_5: f32 = -3.15;
const POKEMON_BOOST_MULTIPLIER_NEG_6: f32 = -3.3;

const POKEMON_FROZEN: f32 = -40.0;
const POKEMON_ASLEEP: f32 = -25.0;
const POKEMON_PARALYZED: f32 = -25.0;
const POKEMON_TOXIC: f32 = -30.0;
const POKEMON_POISONED: f32 = -10.0;
const POKEMON_BURNED: f32 = -25.0;

const LEECH_SEED: f32 = -30.0;
const SUBSTITUTE: f32 = 40.0;
const CONFUSION: f32 = -20.0;

const REFLECT: f32 = 20.0;
const LIGHT_SCREEN: f32 = 20.0;
const STICKY_WEB: f32 = -25.0;
const AURORA_VEIL: f32 = 40.0;
const SAFE_GUARD: f32 = 5.0;
const TAILWIND: f32 = 7.0;

const STEALTH_ROCK: f32 = -10.0;
const SPIKES: f32 = -7.0;
const TOXIC_SPIKES: f32 = -7.0;

fn evaluate_burned(pokemon: &Pokemon) -> f32 {
    // burn is not as punishing in certain situations

    // guts, marvel scale, quick feet will result in a positive evaluation
    match pokemon.ability {
        Abilities::GUTS | Abilities::MARVELSCALE | Abilities::QUICKFEET => return -2.0,
        _ => {}
    }

    let mut multiplier = 0.0;
    let mut iter = pokemon.moves.into_iter();
    for mv in iter {
        if mv.choice.category == MoveCategory::Physical {
            multiplier += 1.0;
        }
    }

    // don't make burn as punishing for special attackers
    if pokemon.special_attack > pokemon.attack {
        multiplier /= 2.0;
    }

    return multiplier * POKEMON_BURNED;
}

fn get_boost_multiplier(boost: i8) -> f32 {
    match boost {
        6 => return POKEMON_BOOST_MULTIPLIER_6,
        5 => return POKEMON_BOOST_MULTIPLIER_5,
        4 => return POKEMON_BOOST_MULTIPLIER_4,
        3 => return POKEMON_BOOST_MULTIPLIER_3,
        2 => return POKEMON_BOOST_MULTIPLIER_2,
        1 => return POKEMON_BOOST_MULTIPLIER_1,
        0 => return POKEMON_BOOST_MULTIPLIER_0,
        -1 => return POKEMON_BOOST_MULTIPLIER_NEG_1,
        -2 => return POKEMON_BOOST_MULTIPLIER_NEG_2,
        -3 => return POKEMON_BOOST_MULTIPLIER_NEG_3,
        -4 => return POKEMON_BOOST_MULTIPLIER_NEG_4,
        -5 => return POKEMON_BOOST_MULTIPLIER_NEG_5,
        -6 => return POKEMON_BOOST_MULTIPLIER_NEG_6,
        _ => return panic!("Invalid boost value: {}", boost),
    }
}

fn evaluate_pokemon(pokemon: &Pokemon) -> f32 {
    let mut score = 0.0;
    score += POKEMON_ALIVE;
    score += POKEMON_HP * pokemon.hp as f32 / pokemon.maxhp as f32;

    score += get_boost_multiplier(pokemon.attack_boost) * POKEMON_ATTACK_BOOST;
    score += get_boost_multiplier(pokemon.defense_boost) * POKEMON_DEFENSE_BOOST;
    score += get_boost_multiplier(pokemon.special_attack_boost) * POKEMON_SPECIAL_ATTACK_BOOST;
    score += get_boost_multiplier(pokemon.special_defense_boost) * POKEMON_SPECIAL_DEFENSE_BOOST;
    score += get_boost_multiplier(pokemon.speed_boost) * POKEMON_SPEED_BOOST;

    match pokemon.status {
        PokemonStatus::Burn => score += evaluate_burned(pokemon),
        PokemonStatus::Freeze => score += POKEMON_FROZEN,
        PokemonStatus::Sleep => score += POKEMON_ASLEEP,
        PokemonStatus::Paralyze => score += POKEMON_PARALYZED,
        PokemonStatus::Toxic => score += POKEMON_TOXIC,
        PokemonStatus::Poison => score += POKEMON_POISONED,
        PokemonStatus::None => {}
    }

    for vs in pokemon.volatile_statuses.iter() {
        match vs {
            PokemonVolatileStatus::LeechSeed => score += LEECH_SEED,
            PokemonVolatileStatus::Substitute => score += SUBSTITUTE,
            PokemonVolatileStatus::Confusion => score += CONFUSION,
            _ => {}
        }
    }

    return score;
}

pub fn evaluate(state: &State) -> f32 {
    let mut score = 0.0;
    let mut side_one_alive_count: f32 = 0.0;
    let mut side_two_alive_count: f32 = 0.0;

    let iter = state.side_one.pokemon.into_iter();
    for pkmn in iter {
        if pkmn.hp > 0 {
            side_one_alive_count += 1.0;
            score += evaluate_pokemon(pkmn);
        }
    }
    let iter = state.side_two.pokemon.into_iter();
    for pkmn in iter {
        if pkmn.hp > 0 {
            // might need something special for randombattles where
            // the pokemon are not revealed
            side_two_alive_count += 1.0;
            score -= evaluate_pokemon(pkmn);
        }
    }

    score += state.side_one.side_conditions.reflect as f32 * REFLECT;
    score += state.side_one.side_conditions.light_screen as f32 * LIGHT_SCREEN;
    score += state.side_one.side_conditions.sticky_web as f32 * STICKY_WEB;
    score += state.side_one.side_conditions.aurora_veil as f32 * AURORA_VEIL;
    score += state.side_one.side_conditions.safeguard as f32 * SAFE_GUARD;
    score += state.side_one.side_conditions.tailwind as f32 * TAILWIND;
    score +=
        state.side_one.side_conditions.stealth_rock as f32 * STEALTH_ROCK * side_one_alive_count;
    score += state.side_one.side_conditions.spikes as f32 * SPIKES * side_one_alive_count;
    score +=
        state.side_one.side_conditions.toxic_spikes as f32 * TOXIC_SPIKES * side_one_alive_count;

    score -= state.side_two.side_conditions.reflect as f32 * REFLECT;
    score -= state.side_two.side_conditions.light_screen as f32 * LIGHT_SCREEN;
    score -= state.side_two.side_conditions.sticky_web as f32 * STICKY_WEB;
    score -= state.side_two.side_conditions.aurora_veil as f32 * AURORA_VEIL;
    score -= state.side_two.side_conditions.safeguard as f32 * SAFE_GUARD;
    score -= state.side_two.side_conditions.tailwind as f32 * TAILWIND;
    score -=
        state.side_two.side_conditions.stealth_rock as f32 * STEALTH_ROCK * side_two_alive_count;
    score -= state.side_two.side_conditions.spikes as f32 * SPIKES * side_two_alive_count;
    score -=
        state.side_two.side_conditions.toxic_spikes as f32 * TOXIC_SPIKES * side_two_alive_count;

    return score;
}
