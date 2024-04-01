use crate::evaluate::evaluate;
use crate::generate_instructions::generate_instructions_from_move_pair;
use crate::state::{MoveChoice, State};

const WIN_BONUS: f32 = 1000.0;

pub fn expectiminimax_search(
    state: &mut State,
    mut depth: i8,
    side_one_options: Vec<MoveChoice>,
    side_two_options: Vec<MoveChoice>,
    ab_prune: bool,
) -> Vec<f32> {
    depth -= 1;
    let num_s1_moves = side_one_options.len();
    let num_s2_moves = side_two_options.len();
    let mut score_lookup: Vec<f32> = Vec::with_capacity(num_s1_moves * num_s2_moves);

    let battle_is_over = state.battle_is_over();
    if battle_is_over != 0.0 {
        for _ in 0..(num_s1_moves * num_s2_moves) {
            score_lookup.push(evaluate(state) + (battle_is_over * WIN_BONUS * depth as f32));
        }
        return score_lookup;
    }

    let mut skip;
    let mut alpha = f32::MIN;
    for side_one_move in side_one_options.iter().as_ref() {
        let mut beta = f32::MAX;
        skip = false;

        for side_two_move in side_two_options.iter().as_ref() {
            if skip {
                score_lookup.push(f32::NAN);
                continue;
            }

            let mut score = 0.0;
            let instructions =
                generate_instructions_from_move_pair(state, &side_one_move, &side_two_move);
            if depth == 0 {
                for instruction in instructions.iter() {
                    state.apply_instructions(&instruction.instruction_list);
                    score += instruction.percentage * evaluate(state) / 100.0;
                    state.reverse_instructions(&instruction.instruction_list);
                }
            } else {
                for instruction in instructions.iter() {
                    state.apply_instructions(&instruction.instruction_list);
                    let (next_turn_side_one_options, next_turn_side_two_options) =
                        state.get_all_options();

                    let next_turn_side_one_options_len = next_turn_side_one_options.len();
                    let next_turn_side_two_options_len = next_turn_side_two_options.len();
                    let (_, safest) = pick_safest(
                        &expectiminimax_search(
                            state,
                            depth,
                            next_turn_side_one_options,
                            next_turn_side_two_options,
                            ab_prune,
                        ),
                        next_turn_side_one_options_len,
                        next_turn_side_two_options_len,
                    );
                    score += instruction.percentage * safest / 100.0;

                    state.reverse_instructions(&instruction.instruction_list);
                }
            }
            score_lookup.push(score);

            if ab_prune {
                if score < beta {
                    beta = score;
                }
                if score <= alpha {
                    skip = true;
                }
            }
        }
        if beta > alpha {
            alpha = beta;
        }
    }
    return score_lookup;
}

pub fn pick_safest(
    score_lookup: &Vec<f32>,
    num_s1_moves: usize,
    num_s2_moves: usize,
) -> (usize, f32) {
    let mut best_worst_case = f32::MIN;
    let mut best_worst_case_s1_index = 0;
    let mut vec_index = 0;

    for s1_index in 0..num_s1_moves {
        let mut worst_case_this_row = f32::MAX;
        for _ in 0..num_s2_moves {
            let score = score_lookup[vec_index];
            vec_index += 1;
            if score < worst_case_this_row {
                worst_case_this_row = score;
            }
        }
        if worst_case_this_row > best_worst_case {
            best_worst_case_s1_index = s1_index;
            best_worst_case = worst_case_this_row;
        }
    }

    return (best_worst_case_s1_index, best_worst_case);
}

fn re_order_moves_for_iterative_deepening(
    last_search_result: &Vec<f32>,
    side_one_options: Vec<MoveChoice>,
    side_two_options: Vec<MoveChoice>,
) -> (Vec<MoveChoice>, Vec<MoveChoice>) {
    let num_s1_moves = side_one_options.len();
    let num_s2_moves = side_two_options.len();
    let mut worst_case_s1_scores: Vec<(MoveChoice, f32)> = vec![];
    let mut vec_index = 0;

    for s1_index in 0..num_s1_moves {
        let mut worst_case_this_row = f32::MAX;
        for _ in 0..num_s2_moves {
            let score = last_search_result[vec_index];
            vec_index += 1;
            if score < worst_case_this_row {
                worst_case_this_row = score;
            }
        }
        worst_case_s1_scores.push((side_one_options[s1_index].clone(), worst_case_this_row));
    }

    worst_case_s1_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let new_s1_vec = worst_case_s1_scores.iter().map(|x| x.0.clone()).collect();

    return (new_s1_vec, side_two_options);
}

pub fn iterative_deepen_expectiminimax(
    state: &mut State,
    depth: i8,
    side_one_options: Vec<MoveChoice>,
    side_two_options: Vec<MoveChoice>,
    ab_prune: bool,
    max_time: std::time::Duration,
) -> (Vec<MoveChoice>, Vec<MoveChoice>, Vec<f32>) {
    let mut result = Vec::new();
    let mut re_ordered_s1_options = side_one_options.clone();
    let mut re_ordered_s2_options = side_two_options.clone();

    let mut start_time = std::time::Instant::now();
    result = expectiminimax_search(state, 1, side_one_options, side_two_options, ab_prune);
    let mut elapsed = start_time.elapsed();

    for i in 2..depth + 1 {
        (re_ordered_s1_options, re_ordered_s2_options) = re_order_moves_for_iterative_deepening(
            &result,
            re_ordered_s1_options,
            re_ordered_s2_options,
        );
        start_time = std::time::Instant::now();
        result = expectiminimax_search(
            state,
            i,
            re_ordered_s1_options.clone(),
            re_ordered_s2_options.clone(),
            ab_prune,
        );
        elapsed = start_time.elapsed();
        if elapsed > std::time::Duration::from_millis(300) {
            break;
        }
    }

    return (re_ordered_s1_options, re_ordered_s2_options, result);
}
