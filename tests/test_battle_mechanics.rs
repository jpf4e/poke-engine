use poke_engine::generate_instructions::generate_instructions_from_move_pair;
use poke_engine::instruction::{
    ApplyVolatileStatusInstruction, BoostInstruction, ChangeSideConditionInstruction,
    ChangeStatusInstruction, DamageInstruction, HealInstruction, Instruction,
    RemoveVolatileStatusInstruction, StateInstructions,
};
use poke_engine::state::{
    PokemonBoostableStat, PokemonSideCondition, PokemonStatus, PokemonVolatileStatus,
    SideReference, State,
};

#[test]
fn test_basic_move_pair_instruction_generation() {
    let mut state = State::default();

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("tackle"),
        String::from("tackle"),
    );

    let expected_instructions = vec![StateInstructions {
        percentage: 100.0,
        instruction_list: vec![
            Instruction::Damage(DamageInstruction {
                side_ref: SideReference::SideOne,
                damage_amount: 48,
            }),
            Instruction::Damage(DamageInstruction {
                side_ref: SideReference::SideTwo,
                damage_amount: 48,
            }),
        ],
    }];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_move_pair_instruction_generation_where_first_move_branches() {
    let mut state = State::default();
    state.side_one.get_active().speed = 101;

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("playrough"),
        String::from("tackle"),
    );

    let expected_instructions = vec![
        StateInstructions {
            percentage: 10.000002,
            instruction_list: vec![Instruction::Damage(DamageInstruction {
                side_ref: SideReference::SideOne,
                damage_amount: 48,
            })],
        },
        StateInstructions {
            percentage: 9.0,
            instruction_list: vec![
                Instruction::Damage(DamageInstruction {
                    side_ref: SideReference::SideTwo,
                    damage_amount: 71,
                }),
                Instruction::Boost(BoostInstruction {
                    side_ref: SideReference::SideTwo,
                    stat: PokemonBoostableStat::Attack,
                    amount: -1,
                }),
                Instruction::Damage(DamageInstruction {
                    side_ref: SideReference::SideOne,
                    // playrough lowered attack means this does less dmg than other branches
                    damage_amount: 33,
                }),
            ],
        },
        StateInstructions {
            percentage: 81.0,
            instruction_list: vec![
                Instruction::Damage(DamageInstruction {
                    side_ref: SideReference::SideTwo,
                    damage_amount: 71,
                }),
                Instruction::Damage(DamageInstruction {
                    side_ref: SideReference::SideOne,
                    damage_amount: 48,
                }),
            ],
        },
    ];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_move_pair_instruction_generation_where_second_move_branches() {
    let mut state = State::default();
    state.side_one.get_active().speed = 50;

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("playrough"),
        String::from("tackle"),
    );

    let expected_instructions = vec![
        StateInstructions {
            percentage: 10.000002,
            instruction_list: vec![Instruction::Damage(DamageInstruction {
                side_ref: SideReference::SideOne,
                damage_amount: 48,
            })],
        },
        StateInstructions {
            percentage: 9.0,
            instruction_list: vec![
                Instruction::Damage(DamageInstruction {
                    side_ref: SideReference::SideOne,
                    damage_amount: 48,
                }),
                Instruction::Damage(DamageInstruction {
                    side_ref: SideReference::SideTwo,
                    damage_amount: 71,
                }),
                Instruction::Boost(BoostInstruction {
                    side_ref: SideReference::SideTwo,
                    stat: PokemonBoostableStat::Attack,
                    amount: -1,
                }),
            ],
        },
        StateInstructions {
            percentage: 81.0,
            instruction_list: vec![
                Instruction::Damage(DamageInstruction {
                    side_ref: SideReference::SideOne,
                    damage_amount: 48,
                }),
                Instruction::Damage(DamageInstruction {
                    side_ref: SideReference::SideTwo,
                    damage_amount: 71,
                }),
            ],
        },
    ];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_basic_flinching_functionality() {
    let mut state = State::default();
    state.side_one.get_active().speed = 150; // faster than side two

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("ironhead"),
        String::from("tackle"),
    );

    let expected_instructions = vec![
        StateInstructions {
            percentage: 30.0000019,
            instruction_list: vec![
                Instruction::Damage(DamageInstruction {
                    side_ref: SideReference::SideTwo,
                    damage_amount: 63,
                }),
                Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    side_ref: SideReference::SideTwo,
                    volatile_status: PokemonVolatileStatus::Flinch,
                }),
                Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    side_ref: SideReference::SideTwo,
                    volatile_status: PokemonVolatileStatus::Flinch,
                }),
            ],
        },
        StateInstructions {
            percentage: 70.0,
            instruction_list: vec![
                Instruction::Damage(DamageInstruction {
                    side_ref: SideReference::SideTwo,
                    damage_amount: 63,
                }),
                Instruction::Damage(DamageInstruction {
                    side_ref: SideReference::SideOne,
                    damage_amount: 48,
                }),
            ],
        },
    ];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_flinching_on_move_that_can_miss() {
    let mut state = State::default();
    state.side_one.get_active().speed = 150; // faster than side two

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("airslash"),
        String::from("tackle"),
    );

    let expected_instructions = vec![
        StateInstructions {
            percentage: 5.000001,
            instruction_list: vec![Instruction::Damage(DamageInstruction {
                side_ref: SideReference::SideOne,
                damage_amount: 48,
            })],
        },
        StateInstructions {
            percentage: 28.5000019,
            instruction_list: vec![
                Instruction::Damage(DamageInstruction {
                    side_ref: SideReference::SideTwo,
                    damage_amount: 60,
                }),
                Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    side_ref: SideReference::SideTwo,
                    volatile_status: PokemonVolatileStatus::Flinch,
                }),
                Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    side_ref: SideReference::SideTwo,
                    volatile_status: PokemonVolatileStatus::Flinch,
                }),
            ],
        },
        StateInstructions {
            percentage: 66.5,
            instruction_list: vec![
                Instruction::Damage(DamageInstruction {
                    side_ref: SideReference::SideTwo,
                    damage_amount: 60,
                }),
                Instruction::Damage(DamageInstruction {
                    side_ref: SideReference::SideOne,
                    damage_amount: 48,
                }),
            ],
        },
    ];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_using_protect_against_damaging_move() {
    let mut state = State::default();

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("protect"),
        String::from("tackle"),
    );

    let expected_instructions = vec![StateInstructions {
        percentage: 100.0,
        instruction_list: vec![
            Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::Protect,
            }),
            Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::Protect,
            }),
            Instruction::ChangeSideCondition(ChangeSideConditionInstruction {
                side_ref: SideReference::SideOne,
                side_condition: PokemonSideCondition::Protect,
                amount: 1,
            }),
        ],
    }];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_crash_move_into_protect() {
    let mut state = State::default();

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("protect"),
        String::from("jumpkick"),
    );

    let expected_instructions = vec![StateInstructions {
        percentage: 100.0,
        instruction_list: vec![
            Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::Protect,
            }),
            Instruction::Damage(DamageInstruction {
                side_ref: SideReference::SideTwo,
                damage_amount: 50,
            }),
            Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::Protect,
            }),
            Instruction::ChangeSideCondition(ChangeSideConditionInstruction {
                side_ref: SideReference::SideOne,
                side_condition: PokemonSideCondition::Protect,
                amount: 1,
            }),
        ],
    }];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_protect_stops_secondaries() {
    let mut state = State::default();

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("protect"),
        String::from("ironhead"),
    );

    let expected_instructions = vec![StateInstructions {
        percentage: 100.0,
        instruction_list: vec![
            Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::Protect,
            }),
            Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::Protect,
            }),
            Instruction::ChangeSideCondition(ChangeSideConditionInstruction {
                side_ref: SideReference::SideOne,
                side_condition: PokemonSideCondition::Protect,
                amount: 1,
            }),
        ],
    }];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_protect_stops_after_damage_hit_callback() {
    let mut state = State::default();
    state.side_one.get_active().item = String::from("dummyitem");

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("protect"),
        String::from("knockoff"),
    );

    let expected_instructions = vec![StateInstructions {
        percentage: 100.0,
        instruction_list: vec![
            Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::Protect,
            }),
            Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::Protect,
            }),
            Instruction::ChangeSideCondition(ChangeSideConditionInstruction {
                side_ref: SideReference::SideOne,
                side_condition: PokemonSideCondition::Protect,
                amount: 1,
            }),
        ],
    }];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_move_that_goes_through_protect() {
    let mut state = State::default();

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("protect"),
        String::from("feint"),
    );

    let expected_instructions = vec![StateInstructions {
        percentage: 100.0,
        instruction_list: vec![
            Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::Protect,
            }),
            Instruction::Damage(DamageInstruction {
                side_ref: SideReference::SideOne,
                damage_amount: 37,
            }),
            Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::Protect,
            }),
            Instruction::ChangeSideCondition(ChangeSideConditionInstruction {
                side_ref: SideReference::SideOne,
                side_condition: PokemonSideCondition::Protect,
                amount: 1,
            }),
        ],
    }];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_using_spikyshield_against_contact_move() {
    let mut state = State::default();

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("spikyshield"),
        String::from("tackle"),
    );

    let expected_instructions = vec![StateInstructions {
        percentage: 100.0,
        instruction_list: vec![
            Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::SpikyShield,
            }),
            Instruction::Heal(HealInstruction {
                side_ref: SideReference::SideTwo,
                heal_amount: -12,
            }),
            Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::SpikyShield,
            }),
            Instruction::ChangeSideCondition(ChangeSideConditionInstruction {
                side_ref: SideReference::SideOne,
                side_condition: PokemonSideCondition::Protect,
                amount: 1,
            }),
        ],
    }];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_spikyshield_recoil_does_not_overkill() {
    let mut state = State::default();
    state.side_two.get_active().hp = 1;

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("spikyshield"),
        String::from("tackle"),
    );

    let expected_instructions = vec![StateInstructions {
        percentage: 100.0,
        instruction_list: vec![
            Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::SpikyShield,
            }),
            Instruction::Heal(HealInstruction {
                side_ref: SideReference::SideTwo,
                heal_amount: -1,
            }),
            Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::SpikyShield,
            }),
            Instruction::ChangeSideCondition(ChangeSideConditionInstruction {
                side_ref: SideReference::SideOne,
                side_condition: PokemonSideCondition::Protect,
                amount: 1,
            }),
        ],
    }];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_spikyshield_does_not_activate_on_non_contact_move() {
    let mut state = State::default();

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("spikyshield"),
        String::from("watergun"),
    );

    let expected_instructions = vec![StateInstructions {
        percentage: 100.0,
        instruction_list: vec![
            Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::SpikyShield,
            }),
            Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::SpikyShield,
            }),
            Instruction::ChangeSideCondition(ChangeSideConditionInstruction {
                side_ref: SideReference::SideOne,
                side_condition: PokemonSideCondition::Protect,
                amount: 1,
            }),
        ],
    }];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_banefulbunker_poisons() {
    let mut state = State::default();

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("banefulbunker"),
        String::from("tackle"),
    );

    let expected_instructions = vec![StateInstructions {
        percentage: 100.0,
        instruction_list: vec![
            Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::BanefulBunker,
            }),
            Instruction::ChangeStatus(ChangeStatusInstruction {
                side_ref: SideReference::SideTwo,
                pokemon_index: 0,
                old_status: PokemonStatus::None,
                new_status: PokemonStatus::Poison,
            }),
            Instruction::Damage(DamageInstruction {
                side_ref: SideReference::SideTwo,
                damage_amount: 12,
            }),
            Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::BanefulBunker,
            }),
            Instruction::ChangeSideCondition(ChangeSideConditionInstruction {
                side_ref: SideReference::SideOne,
                side_condition: PokemonSideCondition::Protect,
                amount: 1,
            }),
        ],
    }];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_banefulbunker_cannot_poison_already_statused_target() {
    let mut state = State::default();
    state.side_two.get_active().status = PokemonStatus::Burn;

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("banefulbunker"),
        String::from("tackle"),
    );

    let expected_instructions = vec![StateInstructions {
        percentage: 100.0,
        instruction_list: vec![
            Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::BanefulBunker,
            }),
            Instruction::Damage(DamageInstruction {
                side_ref: SideReference::SideTwo,
                damage_amount: 6,
            }),
            Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::BanefulBunker,
            }),
            Instruction::ChangeSideCondition(ChangeSideConditionInstruction {
                side_ref: SideReference::SideOne,
                side_condition: PokemonSideCondition::Protect,
                amount: 1,
            }),
        ],
    }];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_silktrap() {
    let mut state = State::default();

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("silktrap"),
        String::from("tackle"),
    );

    let expected_instructions = vec![StateInstructions {
        percentage: 100.0,
        instruction_list: vec![
            Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::SilkTrap,
            }),
            Instruction::Boost(BoostInstruction {
                side_ref: SideReference::SideTwo,
                stat: PokemonBoostableStat::Speed,
                amount: -1,
            }),
            Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                side_ref: SideReference::SideOne,
                volatile_status: PokemonVolatileStatus::SilkTrap,
            }),
            Instruction::ChangeSideCondition(ChangeSideConditionInstruction {
                side_ref: SideReference::SideOne,
                side_condition: PokemonSideCondition::Protect,
                amount: 1,
            }),
        ],
    }];
    assert_eq!(expected_instructions, vec_of_instructions)
}

#[test]
fn test_protect_side_condition_is_removed() {
    let mut state = State::default();
    state.side_one.side_conditions.protect = 1;

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("splash"),
        String::from("splash"),
    );

    let expected_instructions = vec![StateInstructions {
        percentage: 100.0,
        instruction_list: vec![Instruction::ChangeSideCondition(
            ChangeSideConditionInstruction {
                side_ref: SideReference::SideOne,
                side_condition: PokemonSideCondition::Protect,
                amount: -1,
            },
        )],
    }];
    assert_eq!(expected_instructions, vec_of_instructions)
}


#[test]
fn test_protect_for_second_turn_in_a_row() {
    let mut state = State::default();
    state.side_one.side_conditions.protect = 1;

    let vec_of_instructions = generate_instructions_from_move_pair(
        &mut state,
        String::from("protect"),
        String::from("tackle"),
    );

    let expected_instructions = vec![StateInstructions {
        percentage: 100.0,
        instruction_list: vec![
            Instruction::Damage(DamageInstruction {
                side_ref: SideReference::SideOne,
                damage_amount: 48,
            }),
            Instruction::ChangeSideCondition(
                ChangeSideConditionInstruction {
                    side_ref: SideReference::SideOne,
                    side_condition: PokemonSideCondition::Protect,
                    amount: -1,
                },
            )
        ],
    }];
    assert_eq!(expected_instructions, vec_of_instructions)
}
