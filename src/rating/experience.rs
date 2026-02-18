use super::strength::calc_skill_rating;
/// Experience contribution calculations
/// Implements sector-specific experience effects on ratings
use super::types::RatingSector;

/// Calculate the experience rating contribution to a rating sector (Eff(Exp))
///
/// Experience has a non-linear effect modeled by a polynomial:
/// k = -0.00000725 * exp^4 + 0.0005 * exp^3 - 0.01336 * exp^2 + 0.176 * exp
///
/// This value is then multiplied by a sector-specific factor
pub fn calc_experience(sector: RatingSector, skill_value: f64) -> f64 {
    let exp = calc_skill_rating(skill_value);

    // Polynomial formula for experience effect
    let k = -0.00000725 * exp.powi(4) + 0.0005 * exp.powi(3) - 0.01336 * exp.powi(2) + 0.176 * exp;

    // Apply sector-specific multiplier
    let sector_factor = match sector {
        RatingSector::DefenceLeft | RatingSector::DefenceRight => 0.345,
        RatingSector::DefenceCentral => 0.48,
        RatingSector::Midfield => 0.73,
        RatingSector::AttackLeft | RatingSector::AttackRight => 0.375,
        RatingSector::AttackCentral => 0.450,
    };

    k * sector_factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experience_calculation() {
        // Test that experience gives positive contribution
        let exp_value = calc_experience(RatingSector::Midfield, 8.0);
        assert!(exp_value > 0.0);

        // Midfield should have highest factor (0.73)
        let mid_value = calc_experience(RatingSector::Midfield, 8.0);
        let def_value = calc_experience(RatingSector::DefenceLeft, 8.0);
        assert!(mid_value > def_value);
    }

    #[test]
    fn test_experience_sectors() {
        //  Test sector-specific factors
        for sector in RatingSector::all() {
            let value = calc_experience(sector, 10.0);
            assert!(
                value > 0.0,
                "Experience should be positive for {:?}",
                sector
            );
        }
    }
}
