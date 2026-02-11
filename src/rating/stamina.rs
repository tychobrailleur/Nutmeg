use super::strength::calc_skill_rating;
/// Stamina degradation calculations
/// Implements complex stamina effects over match duration (0-120 minutes)
use super::types::TacticType;

/// Calculate the stamina factor for a player at a given match minute
///
/// Stamina degrades over 3 periods:
/// - 0-45 minutes (first half)
/// - 45-90 minutes (second half, with reset at minute 45)
/// - 90-120 minutes (extra time, with reset at minute 90)
///
/// Pressing tactic increases degradation by 10%
///
/// Returns a factor between 0 and 1 representing stamina efficiency
pub fn calc_stamina(stamina: f64, minute: i32, start_minute: i32, tactic_type: TacticType) -> f64 {
    let p = if matches!(tactic_type, TacticType::Pressing) {
        1.1
    } else {
        1.0
    };

    let s = calc_skill_rating(stamina);

    // Base rating and degradation delta depend on stamina skill
    let (r0, delta) = if s < 7.0 {
        let r0 = 102.0 + 23.0 / 7.0 * s;
        let delta = p * (27.0 / 70.0 * s - 5.95);
        (r0, delta)
    } else {
        let r0 = 102.0 + 23.0 + (s - 7.0) * 100.0 / 7.0;
        let delta = -3.25 * p;
        (r0, delta)
    };

    let mut r = r0;

    // First half (0-45)
    let to = minute.min(45);
    if start_minute < to {
        r += (to - start_minute) as f64 * delta / 5.0;
    }

    // Second half (45-90)
    let from = start_minute.max(45);
    if minute >= 45 {
        if start_minute < 45 {
            // Reset at half-time
            r = r.min(r0 + 120.75 - 102.0);
        }
        let to = minute.min(90);
        if from < to {
            r += (to - from) as f64 * delta / 5.0;
        }
    }

    // Extra time (90-120)
    if minute >= 90 {
        let from = start_minute.max(90);
        if start_minute < 90 {
            // Reset at extra time start
            r = r.min(r0 + 127.0 - 120.75);
        }
        if from < minute {
            r += (minute - from) as f64 * delta / 5.0;
        }
    }

    // Clamp to maximum of 1.0
    (r / 100.0).min(1.0)
}

/// Calculate the match average stamina factor
/// Formula fitting the values published by Schum
///
/// This represents the average stamina efficiency over a full 90-minute match
pub fn calc_match_average_stamina_factor(stamina: f64) -> f64 {
    let ret = -0.0033 * stamina * stamina + 0.085 * stamina + 0.51;
    ret.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stamina_at_kickoff() {
        // At minute 0, stamina should be at maximum (close to 1.0)
        let stamina_factor = calc_stamina(8.0, 0, 0, TacticType::Normal);
        assert!(
            stamina_factor > 0.95,
            "Fresh player should have high stamina"
        );
    }

    #[test]
    fn test_stamina_degradation() {
        // Stamina should degrade over time
        let start = calc_stamina(8.0, 0, 0, TacticType::Normal);
        let end = calc_stamina(8.0, 90, 0, TacticType::Normal);
        assert!(end < start, "Stamina should degrade over match");
    }

    #[test]
    fn test_pressing_increases_degradation() {
        // Pressing should cause faster stamina degradation
        let normal = calc_stamina(8.0, 90, 0, TacticType::Normal);
        let pressing = calc_stamina(8.0, 90, 0, TacticType::Pressing);
        assert!(pressing < normal, "Pressing should reduce stamina more");
    }

    #[test]
    fn test_substitution_fresh_stamina() {
        // A player substituted at minute 60 should have fresher stamina
        let full_match = calc_stamina(8.0, 90, 0, TacticType::Normal);
        let subbed_late = calc_stamina(8.0, 90, 60, TacticType::Normal);
        assert!(
            subbed_late > full_match,
            "Substitute should have more stamina"
        );
    }

    #[test]
    fn test_match_average_stamina() {
        let avg = calc_match_average_stamina_factor(8.0);
        assert!(
            avg > 0.5 && avg <= 1.0,
            "Average stamina should be reasonable"
        );
    }
}
