use crate::Attribute;
use crate::rng;
use regex::Regex;

pub fn attr_bonus(value: i32) -> i32 {
    (value-10)/2 
}

pub fn player_hp_per_level(constitution: i32) -> i32 {
    10 + attr_bonus(constitution)
}

pub fn player_hp_at_level(constitution: i32, level: i32) -> i32 {
    10 + player_hp_per_level(constitution) * level
}

pub fn player_xp_for_level(level: i32) -> i32 {
    level * (1000+level*200)
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

pub fn carry_capacity_lbs(strength: &Attribute) -> f32 {
    ((strength.base + strength.modifiers) * 15) as f32
}

/// Parse a dice string into its values
/// eg. 1d10+4 => (1, 10, 4)
pub fn parse_dice_string(dice: &str) -> (i32, i32, i32) {
    lazy_static! {
        static ref DICE_RE: Regex = Regex::new(r"(\d+)d(\d+)([\+\-]\d+)?").unwrap();
    }
    let mut n_dice = 1;
    let mut die_type = 4;
    let mut die_bonus = 0;
    for cap in DICE_RE.captures_iter(dice) {
        if let Some(group) = cap.get(1) {
            n_dice = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(2) {
            die_type = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(3) {
            die_bonus = group.as_str().parse::<i32>().expect("Not a digit");
        }
    }
    (n_dice, die_type, die_bonus)
}

pub fn determine_roll(dice_string: &str) -> i32 {
    let (n_dice, die_type, die_bonus) = parse_dice_string(dice_string);
    rng::roll_dice(n_dice, die_type) + die_bonus
}
