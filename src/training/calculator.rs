use crate::rating::PlayerSkill;

// Base training speeds (approximate weeks to increase 1 level at optimal conditions)
// These are rough community averages for a 17yo.
// Age factor needs to be applied on top.
pub fn get_base_weeks_for_skill(skill: PlayerSkill, current_level: u8) -> f64 {
    // Highly simplified approximation.
    // Real formula is complex and depends on many factors.
    // Base speed for level N -> N+1
    let base = match skill {
        PlayerSkill::Keeper => 4.0,
        PlayerSkill::Defending => 5.0,
        PlayerSkill::Playmaking => 5.5, // Slower
        PlayerSkill::Winger => 4.5,
        PlayerSkill::Scoring => 5.5,
        PlayerSkill::Passing => 4.0, // Fastest secondary
        PlayerSkill::SetPieces => 2.0,
        _ => 5.0,
    };

    // Level factor: It gets harder as you go up.
    // Approximately +5-10% per level.
    let level_factor = 1.0 + (current_level as f64 * 0.08);

    base * level_factor
}

pub fn get_age_factor(age_years: u8) -> f64 {
    // 17yo = 1.0 (baseline)
    // 18yo = 1.08
    // 19yo = 1.16
    // ...
    // Non-linear, increases faster with age.
    if age_years < 17 {
        return 1.0;
    }
    let age_diff = (age_years - 17) as f64;
    1.0 + (age_diff * 0.10) // Approx 10% slower per year
}

pub fn calculate_training_progress(
    current_skill: f64,
    skill_type: PlayerSkill,
    age_years: u8,
    training_intensity: f64, // 0.0 to 1.0 (usually 1.0)
    stamina_share: f64,      // 0.0 to 1.0 (usually 0.10 - 0.15)
) -> f64 {
    let current_level_int = current_skill.floor() as u8;
    let weeks_needed =
        get_base_weeks_for_skill(skill_type, current_level_int) * get_age_factor(age_years);

    // Amount of skill gained in one week (1.0 = one full level)
    // We assume 100% training position (e.g., IM in PM).
    let weekly_gain = (1.0 / weeks_needed) * training_intensity;

    // Stamina reduction (training efficiency reduced by stamina share)
    // Actually, stamina share *replaces* skill training.
    let effective_gain = weekly_gain * (1.0 - stamina_share);

    effective_gain
}
