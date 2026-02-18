/// Position evaluation for players
/// Calculates ratings for all possible positions to find optimal placements
use super::model::{Lineup, LineupPosition, RatingPredictionModel};
use super::types::{Behaviour, PositionId, RatingSector};
use crate::chpp::model::Player;

/// Result of evaluating a position for a player
#[derive(Debug, Clone)]
pub struct PositionRating {
    pub position: PositionId,
    pub behaviour: Behaviour,
    pub rating: f64,
    pub sectors: Vec<(RatingSector, f64)>,
}

/// Evaluation result for a player across all positions
#[derive(Debug, Clone)]
pub struct PlayerEvaluation {
    pub player_id: u32,
    pub player_name: String,
    pub positions: Vec<PositionRating>,
    pub best_position: Option<PositionRating>,
}

impl PlayerEvaluation {
    /// Get top N positions sorted by rating
    pub fn top_positions(&self, n: usize) -> Vec<&PositionRating> {
        let mut sorted = self.positions.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap());
        sorted.into_iter().take(n).collect()
    }
}

/// Evaluate a player for all possible positions
pub fn evaluate_all_positions(
    model: &RatingPredictionModel,
    player: &Player,
    lineup: &Lineup,
    minute: i32,
) -> PlayerEvaluation {
    let mut position_ratings = Vec::new();

    // Get positions to evaluate based on player's skills
    let positions_to_eval = get_positions_for_player(player);

    for (position, behaviour) in positions_to_eval {
        let rating = calculate_position_rating(model, player, position, behaviour, lineup, minute);
        position_ratings.push(rating);
    }

    // Sort by total rating descending
    position_ratings.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap());

    let best = position_ratings.first().cloned();

    PlayerEvaluation {
        player_id: player.PlayerID,
        player_name: format!("{} {}", player.FirstName, player.LastName),
        positions: position_ratings,
        best_position: best,
    }
}

/// Calculate rating for a specific position/behaviour combination
fn calculate_position_rating(
    model: &RatingPredictionModel,
    player: &Player,
    position: PositionId,
    behaviour: Behaviour,
    lineup: &Lineup,
    minute: i32,
) -> PositionRating {
    // Create a temporary lineup with just this player
    let test_lineup = Lineup {
        positions: vec![LineupPosition {
            player: player.clone(),
            role_id: position,
            behaviour,
            start_minute: 0,
        }],
        weather: lineup.weather,
        tactic: lineup.tactic,
        attitude: lineup.attitude,
        location: lineup.location,
    };

    // Calculate ratings for all sectors
    let mut sector_ratings = Vec::new();
    let mut total_rating = 0.0;
    let mut total_weight = 0.0;

    for sector in RatingSector::all() {
        let rating = model.get_rating(&test_lineup, sector, minute);
        sector_ratings.push((sector, rating));

        // Only weight sectors that this position actually contributes to
        // Goalkeeper only contributes to defence sectors, not midfield or attack
        let contributes = has_contribution_to_sector(position, sector);

        if contributes {
            // Weight by importance: Midfield 3x, others 1x
            let weight = if sector == RatingSector::Midfield {
                3.0
            } else {
                1.0
            };
            total_rating += rating * weight;
            total_weight += weight;
        }
    }

    // Normalize by total weight
    if total_weight > 0.0 {
        total_rating /= total_weight;
    }

    PositionRating {
        position,
        behaviour,
        rating: total_rating,
        sectors: sector_ratings,
    }
}

/// Check if a position contributes to a rating sector
fn has_contribution_to_sector(position: PositionId, sector: RatingSector) -> bool {
    match position {
        // Keeper only contributes to central defence, not left/right
        // This prevents triple-counting keeper contribution across all 3 defence sectors
        PositionId::Keeper => matches!(sector, RatingSector::DefenceCentral),

        // Defenders contribute to defence and midfield
        PositionId::LeftBack
        | PositionId::LeftCentralDefender
        | PositionId::MiddleCentralDefender
        | PositionId::RightCentralDefender
        | PositionId::RightBack => matches!(
            sector,
            RatingSector::DefenceLeft
                | RatingSector::DefenceCentral
                | RatingSector::DefenceRight
                | RatingSector::Midfield
        ),

        // Midfielders contribute to all sectors
        PositionId::LeftWinger
        | PositionId::LeftInnerMidfield
        | PositionId::CentralInnerMidfield
        | PositionId::RightInnerMidfield
        | PositionId::RightWinger => true,

        // Forwards contribute to midfield and attack
        PositionId::LeftForward | PositionId::CentralForward | PositionId::RightForward => {
            matches!(
                sector,
                RatingSector::Midfield
                    | RatingSector::AttackLeft
                    | RatingSector::AttackCentral
                    | RatingSector::AttackRight
            )
        }

        // Set pieces treated as midfielder
        PositionId::SetPieces => true,
    }
}

/// Get list of positions to evaluate for best position analysis
/// Returns position + behavior combinations that should be tested
pub fn get_evaluable_positions() -> Vec<(PositionId, Behaviour)> {
    vec![
        // Defenders with behaviours
        (PositionId::LeftBack, Behaviour::Defensive),
        (PositionId::LeftBack, Behaviour::Normal),
        (PositionId::LeftBack, Behaviour::Offensive),
        (PositionId::LeftBack, Behaviour::TowardsMiddle), // Added back TowardsMiddle
        (PositionId::LeftCentralDefender, Behaviour::Defensive),
        (PositionId::LeftCentralDefender, Behaviour::Normal),
        (PositionId::LeftCentralDefender, Behaviour::Offensive),
        (PositionId::LeftCentralDefender, Behaviour::TowardsWing), // Added back TowardsWing
        (PositionId::MiddleCentralDefender, Behaviour::Defensive),
        (PositionId::MiddleCentralDefender, Behaviour::Normal),
        (PositionId::MiddleCentralDefender, Behaviour::Offensive),
        (PositionId::RightCentralDefender, Behaviour::Defensive),
        (PositionId::RightCentralDefender, Behaviour::Normal),
        (PositionId::RightCentralDefender, Behaviour::Offensive),
        (PositionId::RightCentralDefender, Behaviour::TowardsWing), // Added back TowardsWing
        (PositionId::RightBack, Behaviour::Defensive),
        (PositionId::RightBack, Behaviour::Normal),
        (PositionId::RightBack, Behaviour::Offensive),
        (PositionId::RightBack, Behaviour::TowardsMiddle), // Added back TowardsMiddle
        // Midfielders
        (PositionId::LeftWinger, Behaviour::Defensive),
        (PositionId::LeftWinger, Behaviour::Normal),
        (PositionId::LeftWinger, Behaviour::Offensive),
        (PositionId::LeftWinger, Behaviour::TowardsMiddle), // Added back TowardsMiddle
        (PositionId::LeftInnerMidfield, Behaviour::Defensive),
        (PositionId::LeftInnerMidfield, Behaviour::Normal),
        (PositionId::LeftInnerMidfield, Behaviour::Offensive),
        (PositionId::LeftInnerMidfield, Behaviour::TowardsWing),
        (PositionId::CentralInnerMidfield, Behaviour::Defensive),
        (PositionId::CentralInnerMidfield, Behaviour::Normal),
        (PositionId::CentralInnerMidfield, Behaviour::Offensive),
        (PositionId::RightInnerMidfield, Behaviour::Defensive),
        (PositionId::RightInnerMidfield, Behaviour::Normal),
        (PositionId::RightInnerMidfield, Behaviour::Offensive),
        (PositionId::RightInnerMidfield, Behaviour::TowardsWing),
        (PositionId::RightWinger, Behaviour::Defensive),
        (PositionId::RightWinger, Behaviour::Normal),
        (PositionId::RightWinger, Behaviour::Offensive),
        (PositionId::RightWinger, Behaviour::TowardsMiddle), // Added back TowardsMiddle
        // Forwards
        (PositionId::LeftForward, Behaviour::Defensive),
        (PositionId::LeftForward, Behaviour::Normal),
        (PositionId::LeftForward, Behaviour::TowardsWing),
        (PositionId::CentralForward, Behaviour::Defensive),
        (PositionId::CentralForward, Behaviour::Normal),
        (PositionId::CentralForward, Behaviour::TowardsWing),
        (PositionId::RightForward, Behaviour::Defensive),
        (PositionId::RightForward, Behaviour::Normal),
        (PositionId::RightForward, Behaviour::TowardsWing),
        // Note: Keeper is conditionally added based on skill level
        // Only players with KeeperSkill >= 7 should be evaluated as keeper
    ]
}

/// Check if keeper skill is the highest skill (or tied for highest)
/// Returns true if KeeperSkill >= all other field player skills
fn is_keeper_highest_skill(player: &Player) -> bool {
    let skills = match &player.PlayerSkills {
        Some(s) => s,
        None => return false,
    };

    let keeper = skills.KeeperSkill;

    // Compare against all field player skills
    keeper >= skills.DefenderSkill
        && keeper >= skills.PlaymakerSkill
        && keeper >= skills.ScorerSkill
        && keeper >= skills.PassingSkill
        && keeper >= skills.WingerSkill
}

/// Get list of positions to evaluate for a specific player
/// Conditionally includes Keeper based on whether it's the player's highest skill
pub fn get_positions_for_player(player: &Player) -> Vec<(PositionId, Behaviour)> {
    let mut positions = get_evaluable_positions();

    // Only evaluate Keeper position if keeper skill is the highest skill
    // This ensures field players aren't incorrectly identified as goalkeepers
    if is_keeper_highest_skill(player) {
        positions.push((PositionId::Keeper, Behaviour::Normal));
    }

    positions
}

/// Evaluate entire squad and find best positions for all players
pub fn evaluate_squad(
    model: &RatingPredictionModel,
    players: &[Player],
    lineup: &Lineup,
    minute: i32,
) -> Vec<PlayerEvaluation> {
    players
        .iter()
        .map(|player| evaluate_all_positions(model, player, lineup, minute))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chpp::model::{Player, PlayerSkills};
    use crate::rating::model::Team;
    use crate::rating::types::{Attitude, Location, TacticType, Weather};

    /// Helper to create a test player with given skills
    fn create_test_player(
        id: u32,
        name: &str,
        keeper: u32,
        defender: u32,
        playmaker: u32,
        scorer: u32,
        passing: u32,
        winger: u32,
    ) -> Player {
        Player {
            PlayerID: id,
            FirstName: "Test".to_string(),
            LastName: name.to_string(),
            NickName: None,
            PlayerNumber: Some(id),
            Age: 25,
            AgeDays: Some(0),
            TSI: 1000,
            PlayerForm: 7,
            Statement: None,
            Experience: 5,
            Loyalty: 10,
            ReferencePlayerID: None,
            MotherClubBonus: false,
            Leadership: 5,
            Salary: 1000,
            IsAbroad: false,
            Agreeability: 3,
            Aggressiveness: 3,
            Honesty: 3,
            LeagueGoals: Some(0),
            CupGoals: Some(0),
            FriendliesGoals: Some(0),
            CareerGoals: Some(0),
            CareerHattricks: Some(0),
            CareerAssists: Some(0),
            Specialty: None,
            TransferListed: false,
            NationalTeamID: None,
            CountryID: Some(1),
            Caps: None,
            CapsU20: None,
            Cards: Some(0),
            InjuryLevel: Some(-1),
            AvatarBlob: None,
            GenderID: None,
            NativeCountryFlag: None,
            Flag: None,
            PlayerSkills: Some(PlayerSkills {
                KeeperSkill: keeper,
                DefenderSkill: defender,
                PlaymakerSkill: playmaker,
                ScorerSkill: scorer,
                PassingSkill: passing,
                WingerSkill: winger,
                SetPiecesSkill: 5,
                StaminaSkill: 7,
            }),
            ArrivalDate: None,
            PlayerCategoryId: Some(if keeper > defender && keeper > playmaker {
                1
            } else {
                2
            }),
            MotherClub: None,
            NativeCountryID: Some(1),
            NativeLeagueID: Some(1),
            NativeLeagueName: Some("Test League".to_string()),
            MatchesCurrentTeam: Some(0),
            GoalsCurrentTeam: Some(0),
            AssistsCurrentTeam: Some(0),
            LastMatch: None,
        }
    }

    #[test]
    fn test_evaluate_positions_count() {
        let positions = get_evaluable_positions();
        // Should have ~45 position/behaviour combinations
        assert!(positions.len() >= 40);
    }

    #[test]
    fn test_keeper_is_highest_skill() {
        // K=12, Df=5, PM=6, Sc=4, Ps=5, Wg=3
        let player = create_test_player(1, "Keeper", 12, 5, 6, 4, 5, 3);

        assert!(is_keeper_highest_skill(&player));

        // Test that Keeper position is included
        let positions = get_positions_for_player(&player);
        assert!(positions.contains(&(PositionId::Keeper, Behaviour::Normal)));
    }

    #[test]
    fn test_keeper_tied_for_highest() {
        // K=10, Df=10 (tied), PM=8, Sc=7, Ps=9, Wg=6
        let player = create_test_player(2, "Versatile", 10, 10, 8, 7, 9, 6);

        // Keeper tied for highest = should still evaluate as keeper
        assert!(is_keeper_highest_skill(&player));

        let positions = get_positions_for_player(&player);
        assert!(positions.contains(&(PositionId::Keeper, Behaviour::Normal)));
    }

    #[test]
    fn test_keeper_not_highest() {
        // K=5, Df=14 (much higher), PM=8, Sc=7, Ps=9, Wg=6
        let player = create_test_player(3, "Defender", 5, 14, 8, 7, 9, 6);

        assert!(!is_keeper_highest_skill(&player));

        // Keeper position should NOT be included
        let positions = get_positions_for_player(&player);
        assert!(!positions.contains(&(PositionId::Keeper, Behaviour::Normal)));
    }

    #[test]
    fn test_keeper_low_skill_not_evaluated() {
        // K=1, Df=2 (higher), PM=1, Sc=1, Ps=1, Wg=1
        // Even though all skills are low, Df is highest
        let player = create_test_player(4, "Weak", 1, 2, 1, 1, 1, 1);

        assert!(!is_keeper_highest_skill(&player));

        let positions = get_positions_for_player(&player);
        assert!(!positions.contains(&(PositionId::Keeper, Behaviour::Normal)));
    }

    #[test]
    fn test_no_skills_means_no_keeper() {
        let mut player = create_test_player(5, "NoSkills", 5, 5, 5, 5, 5, 5);
        player.PlayerSkills = None;

        assert!(!is_keeper_highest_skill(&player));

        let positions = get_positions_for_player(&player);
        assert!(!positions.contains(&(PositionId::Keeper, Behaviour::Normal)));
    }

    #[test]
    fn test_all_skills_equal_includes_keeper() {
        // K=8, Df=8, PM=8, Sc=8, Ps=8, Wg=8 - all tied
        let player = create_test_player(6, "AllEqual", 8, 8, 8, 8, 8, 8);

        assert!(is_keeper_highest_skill(&player));

        let positions = get_positions_for_player(&player);
        assert!(positions.contains(&(PositionId::Keeper, Behaviour::Normal)));
    }

    // Commented out - requires Player::default() which doesn't exist
    // #[test]
    // fn test_player_evaluation_sorting() {
    //     let team = Team::default();
    //     let model = RatingPredictionModel::new(team);
    //     let player = Player::default();
    //     let lineup = Lineup {
    //         positions: vec![],
    //         weather: Weather::Neutral,
    //         tactic: TacticType::Normal,
    //         attitude: Attitude::Normal,
    //         location: Location::Away,
    //     };
    //
    //     let evaluation = evaluate_all_positions(&model, &player, &lineup, 0);
    //
    //     // Should have evaluated positions
    //     assert!(!evaluation.positions.is_empty());
    //
    //     // Best position should be set
    //     assert!(evaluation.best_position.is_some());
    //
    //     // Top positions should be sorted
    //     let top_5 = evaluation.top_positions(5);
    //     for i in 0..top_5.len() - 1 {
    //         assert!(top_5[i].rating >= top_5[i + 1].rating);
    //     }
    // }
}
