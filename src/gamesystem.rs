pub fn attr_bonus(value: i32) -> i32 {
    (value-10)/2 
}

pub fn player_hp_per_level(constitution: i32) -> i32 {
    10 + attr_bonus(constitution)
}

pub fn player_hp_at_level(constitution: i32, level: i32) -> i32 {
    10 + player_hp_per_level(constitution) * level
}

pub fn npc_hp(constitution: i32, level: i32) -> i32 {
    let mut total = 1;
    for _ in 0..level {
        total += i32::max(1, 8 + attr_bonus(constitution));
    }
    total
}

pub fn mana_per_level(intelligence: i32) -> i32 {
    4 + attr_bonus(intelligence)
}

pub fn mana_at_level(intelligence: i32, level: i32) -> i32 {
    mana_per_level(intelligence) * level
}
