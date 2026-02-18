use super::contribution::calc_contribution;
use super::experience::calc_experience;
use super::parameters::get_overcrowding_penalty;
use super::stamina::calc_stamina;
/// Rating prediction model
/// Main entry point for rating calculations using sector-based analysis
use super::types::*;
use super::weather::calc_weather;
use crate::chpp::model::Player;

/// Team information needed for rating calculations
#[derive(Debug, Clone)]
pub struct Team {
    pub team_spirit: f64,    // 0-20 scale
    pub confidence: f64,     // 0-20 scale
    pub coach_modifier: i32, // -10 (defensive) to +10 (offensive)
}

impl Default for Team {
    fn default() -> Self {
        Self {
            team_spirit: 5.0,
            confidence: 5.0,
            coach_modifier: 0,
        }
    }
}

/// A player in a specific lineup position
#[derive(Debug, Clone)]
pub struct LineupPosition {
    pub player: Player,
    pub role_id: PositionId,
    pub behaviour: Behaviour,
    pub start_minute: i32,
}

/// Match lineup configuration
#[derive(Debug, Clone)]
pub struct Lineup {
    pub positions: Vec<LineupPosition>,
    pub weather: Weather,
    pub tactic: TacticType,
    pub attitude: Attitude,
    pub location: Location,
}

impl Lineup {
    /// Get all players active at a given minute (includes keeper)
    pub fn get_active_players(&self, minute: i32) -> Vec<&LineupPosition> {
        self.positions
            .iter()
            .filter(|p| p.start_minute <= minute)
            .collect()
    }

    /// Count players in a specific sector
    pub fn count_players_in_sector(&self, sector: Sector) -> usize {
        self.positions
            .iter()
            .filter(|p| p.role_id.sector() == sector)
            .count()
    }
}

/// Main rating prediction model
pub struct RatingPredictionModel {
    team: Team,
}

impl RatingPredictionModel {
    pub fn new(team: Team) -> Self {
        Self { team }
    }

    /// Get rating for a sector at a specific minute
    pub fn get_rating(&self, lineup: &Lineup, sector: RatingSector, minute: i32) -> f64 {
        self.calc_sector_rating(lineup, sector, minute)
    }

    /// Calculate the rating for a specific sector
    fn calc_sector_rating(&self, lineup: &Lineup, sector: RatingSector, minute: i32) -> f64 {
        let mut rating = 0.0;

        // Sum contributions from all active players (including keeper)
        for position in lineup.get_active_players(minute) {
            // Get overcrowding penalty for this player's sector
            let sector_count = lineup.count_players_in_sector(position.role_id.sector());
            let overcrowding = get_overcrowding_penalty(sector_count, position.role_id.sector());

            // Calculate base contribution
            let mut contrib = calc_contribution(
                &position.player,
                position.role_id,
                position.behaviour,
                sector,
            );

            // Apply overcrowding penalty
            contrib *= overcrowding;

            // Add experience effect
            let exp_skill = position.player.Experience as f64;
            contrib += calc_experience(sector, exp_skill);

            // Get player specialty for weather
            let specialty = get_player_specialty(&position.player);

            // Apply weather effect
            contrib *= calc_weather(specialty, lineup.weather);

            // Apply stamina effect
            let stamina_skill = position
                .player
                .PlayerSkills
                .as_ref()
                .map(|ps| ps.StaminaSkill)
                .unwrap_or(0) as f64;
            contrib *= calc_stamina(stamina_skill, minute, position.start_minute, lineup.tactic);

            rating += contrib;
        }

        // Apply sector-specific factors
        rating *= self.calc_sector_factors(lineup, sector);

        // Transform to rating scale
        calc_rating_sector_scale(sector, rating)
    }

    /// Calculate sector-specific multiplication factors
    fn calc_sector_factors(&self, lineup: &Lineup, sector: RatingSector) -> f64 {
        let mut factor = 1.0;

        match sector {
            RatingSector::Midfield => {
                // Team spirit effect (Midfield only)
                factor *= calc_team_spirit(self.team.team_spirit);

                // Attitude effects
                match lineup.attitude {
                    Attitude::PlayItCool => factor *= 0.83945,
                    Attitude::MatchOfTheSeason => factor *= 1.1149,
                    _ => {}
                }

                // Location effects
                match lineup.location {
                    Location::Home => factor *= 1.19892,
                    Location::AwayDerby => factor *= 1.11493,
                    _ => {}
                }

                // Tactic effects
                match lineup.tactic {
                    TacticType::CounterAttacks => factor *= 0.93,
                    TacticType::LongShots => factor *= 0.96,
                    _ => {}
                }
            }
            RatingSector::DefenceLeft
            | RatingSector::DefenceCentral
            | RatingSector::DefenceRight => {
                // Coach modifier
                factor *= calc_trainer(sector, self.team.coach_modifier);

                // Tactic effects
                match lineup.tactic {
                    TacticType::PlayCreatively => factor *= 0.93,
                    _ => {}
                }

                if matches!(
                    sector,
                    RatingSector::DefenceLeft | RatingSector::DefenceRight
                ) {
                    if matches!(lineup.tactic, TacticType::AttackInTheMiddle) {
                        factor *= 0.85;
                    }
                } else if matches!(sector, RatingSector::DefenceCentral) {
                    if matches!(lineup.tactic, TacticType::AttackOnWings) {
                        factor *= 0.85;
                    }
                }
            }
            RatingSector::AttackLeft | RatingSector::AttackCentral | RatingSector::AttackRight => {
                // Confidence effect (Attack only)
                factor *= calc_confidence(self.team.confidence);

                // Coach modifier
                factor *= calc_trainer(sector, self.team.coach_modifier);

                // Tactic effects
                if matches!(lineup.tactic, TacticType::LongShots) {
                    factor *= 0.96;
                }
            }
        }

        factor
    }

    /// Get average rating over a match duration
    pub fn get_average_rating(&self, lineup: &Lineup, sector: RatingSector, duration: i32) -> f64 {
        // Sample rating at beginning, middle, and end
        let start = self.get_rating(lineup, sector, 0);
        let middle = self.get_rating(lineup, sector, duration / 2);
        let end = self.get_rating(lineup, sector, duration);

        // Simple average (could be more sophisticated with integration)
        (start + 2.0 * middle + end) / 4.0
    }

    /// Calculate HatStats (weighted sum of all sectors)
    /// Formula: 4 * (3×Midfield + all other sectors)
    pub fn calc_hatstats(&self, lineup: &Lineup, minute: i32) -> f64 {
        let mut hatstats = 0.0;

        hatstats += 3.0 * self.get_rating(lineup, RatingSector::Midfield, minute);
        hatstats += self.get_rating(lineup, RatingSector::DefenceLeft, minute);
        hatstats += self.get_rating(lineup, RatingSector::DefenceCentral, minute);
        hatstats += self.get_rating(lineup, RatingSector::DefenceRight, minute);
        hatstats += self.get_rating(lineup, RatingSector::AttackLeft, minute);
        hatstats += self.get_rating(lineup, RatingSector::AttackCentral, minute);
        hatstats += self.get_rating(lineup, RatingSector::AttackRight, minute);

        4.0 * hatstats
    }
}

/// Get player's specialty
fn get_player_specialty(player: &Player) -> Specialty {
    match player.Specialty {
        Some(1) => Specialty::Technical,
        Some(2) => Specialty::Quick,
        Some(3) => Specialty::Powerful,
        Some(4) => Specialty::Unpredictable,
        Some(5) => Specialty::Head,
        Some(6) => Specialty::Regainer,
        Some(8) => Specialty::Support,
        _ => Specialty::NoSpecialty,
    }
}

/// Calculate team spirit factor (Midfield only)
/// Formula: 0.1 + 0.425 * sqrt(team_spirit)
fn calc_team_spirit(team_spirit: f64) -> f64 {
    0.1 + 0.425 * team_spirit.sqrt()
}

/// Calculate confidence factor (Attack only)
/// Formula: 0.8 + 0.05 * (confidence + 0.5)
fn calc_confidence(confidence: f64) -> f64 {
    0.8 + 0.05 * (confidence + 0.5)
}

/// Calculate coach modifier factor
fn calc_trainer(sector: RatingSector, coach_modifier: i32) -> f64 {
    match sector {
        RatingSector::DefenceLeft | RatingSector::DefenceRight | RatingSector::DefenceCentral => {
            if coach_modifier <= 0 {
                // Balanced or Defensive
                1.02 - coach_modifier as f64 * (1.15 - 1.02) / 10.0
            } else {
                // Offensive
                1.02 - coach_modifier as f64 * (1.02 - 0.9) / 10.0
            }
        }
        RatingSector::AttackCentral | RatingSector::AttackLeft | RatingSector::AttackRight => {
            if coach_modifier <= 0 {
                // Balanced or Defensive
                1.02 - coach_modifier as f64 * (0.9 - 1.02) / 10.0
            } else {
                // Offensive
                1.02 - coach_modifier as f64 * (1.02 - 1.1) / 10.0
            }
        }
        _ => 1.0,
    }
}

/// Transform rating value to rating scale
fn calc_rating_sector_scale(sector: RatingSector, value: f64) -> f64 {
    if value > 0.0 {
        let scale_factor = match sector {
            RatingSector::Midfield => 0.312,
            RatingSector::DefenceLeft | RatingSector::DefenceRight => 0.834,
            RatingSector::DefenceCentral => 0.501,
            RatingSector::AttackCentral => 0.513,
            RatingSector::AttackLeft | RatingSector::AttackRight => 0.615,
        };
        // Formula: (value * scale_factor)^1.2 + 1.0
        // Note: Removed /4.0 division - was causing incorrect scaling
        (value * scale_factor).powf(1.2) + 1.0
    } else {
        0.75 // Minimum rating
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_spirit_calculation() {
        assert!((calc_team_spirit(5.0) - 1.049).abs() < 0.01);
        assert!((calc_team_spirit(10.0) - 1.443).abs() < 0.01);
    }

    #[test]
    fn test_confidence_calculation() {
        assert!((calc_confidence(5.0) - 1.075).abs() < 0.01);
        assert!((calc_confidence(10.0) - 1.325).abs() < 0.01);
    }

    #[test]
    fn test_rating_sector_scale() {
        // Test that positive values scale correctly
        let rating = calc_rating_sector_scale(RatingSector::Midfield, 10.0);
        assert!(rating > 1.0);

        // Test minimum rating
        let min_rating = calc_rating_sector_scale(RatingSector::Midfield, 0.0);
        assert_eq!(min_rating, 0.75);
    }

    #[test]
    fn test_basic_model_creation() {
        let team = Team::default();
        let model = RatingPredictionModel::new(team);

        let lineup = Lineup {
            positions: vec![],
            weather: Weather::Neutral,
            tactic: TacticType::Normal,
            attitude: Attitude::Normal,
            location: Location::Away,
        };

        let rating = model.get_rating(&lineup, RatingSector::Midfield, 0);
        assert_eq!(rating, 0.75); // Empty lineup = minimum rating
    }
}
