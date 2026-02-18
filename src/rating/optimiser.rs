use super::model::{Lineup, LineupPosition};
use super::position_eval::{calculate_position_rating, evaluate_all_positions, PositionRating};
use super::types::{Attitude, Behaviour, Location, PositionId, RatingSector, TacticType, Weather};
use crate::chpp::model::Player;
use crate::rating::RatingPredictionModel;
use std::collections::{HashMap, HashSet};

/// Standard formations in Hattrick
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Formation {
    F550,
    F541,
    F532,
    F523,
    F451,
    F442,
    F433,
    F352,
    F343,
    F253,
}

impl Formation {
    pub fn all() -> [Formation; 10] {
        [
            Formation::F550,
            Formation::F541,
            Formation::F532,
            Formation::F523,
            Formation::F451,
            Formation::F442,
            Formation::F433,
            Formation::F352,
            Formation::F343,
            Formation::F253,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Formation::F550 => "5-5-0",
            Formation::F541 => "5-4-1",
            Formation::F532 => "5-3-2",
            Formation::F523 => "5-2-3",
            Formation::F451 => "4-5-1",
            Formation::F442 => "4-4-2",
            Formation::F433 => "4-3-3",
            Formation::F352 => "3-5-2",
            Formation::F343 => "3-4-3",
            Formation::F253 => "2-5-3",
        }
    }

    /// Get standard slots for this formation
    /// Uses standard Hattrick slots (e.g. 4-4-2 uses standard back 4, 2 wingers, 2 IMs, 2 FWs)
    pub fn get_slots(&self) -> Vec<PositionId> {
        let mut slots = vec![PositionId::Keeper];

        match self {
            Formation::F550 => {
                slots.extend_from_slice(&[
                    PositionId::RightBack,
                    PositionId::RightCentralDefender,
                    PositionId::MiddleCentralDefender,
                    PositionId::LeftCentralDefender,
                    PositionId::LeftBack,
                    PositionId::RightWinger,
                    PositionId::RightInnerMidfield,
                    PositionId::CentralInnerMidfield,
                    PositionId::LeftInnerMidfield,
                    PositionId::LeftWinger,
                ]);
            }
            Formation::F541 => {
                slots.extend_from_slice(&[
                    PositionId::RightBack,
                    PositionId::RightCentralDefender,
                    PositionId::MiddleCentralDefender,
                    PositionId::LeftCentralDefender,
                    PositionId::LeftBack,
                    PositionId::RightWinger,
                    PositionId::RightInnerMidfield,
                    PositionId::LeftInnerMidfield,
                    PositionId::LeftWinger,
                    PositionId::CentralForward,
                ]);
            }
            Formation::F532 => {
                slots.extend_from_slice(&[
                    PositionId::RightBack,
                    PositionId::RightCentralDefender,
                    PositionId::MiddleCentralDefender,
                    PositionId::LeftCentralDefender,
                    PositionId::LeftBack,
                    PositionId::RightInnerMidfield,
                    PositionId::CentralInnerMidfield,
                    PositionId::LeftInnerMidfield,
                    PositionId::RightForward,
                    PositionId::LeftForward,
                ]);
            }
            Formation::F523 => {
                slots.extend_from_slice(&[
                    PositionId::RightBack,
                    PositionId::RightCentralDefender,
                    PositionId::MiddleCentralDefender,
                    PositionId::LeftCentralDefender,
                    PositionId::LeftBack,
                    PositionId::RightInnerMidfield,
                    PositionId::LeftInnerMidfield,
                    PositionId::RightForward,
                    PositionId::CentralForward,
                    PositionId::LeftForward,
                ]);
            }
            Formation::F451 => {
                slots.extend_from_slice(&[
                    PositionId::RightBack,
                    PositionId::RightCentralDefender,
                    PositionId::LeftCentralDefender,
                    PositionId::LeftBack,
                    PositionId::RightWinger,
                    PositionId::RightInnerMidfield,
                    PositionId::CentralInnerMidfield,
                    PositionId::LeftInnerMidfield,
                    PositionId::LeftWinger,
                    PositionId::CentralForward,
                ]);
            }
            Formation::F442 => {
                slots.extend_from_slice(&[
                    PositionId::RightBack,
                    PositionId::RightCentralDefender,
                    PositionId::LeftCentralDefender,
                    PositionId::LeftBack,
                    PositionId::RightWinger,
                    PositionId::RightInnerMidfield,
                    PositionId::LeftInnerMidfield,
                    PositionId::LeftWinger,
                    PositionId::RightForward,
                    PositionId::LeftForward,
                ]);
            }
            Formation::F433 => {
                slots.extend_from_slice(&[
                    PositionId::RightBack,
                    PositionId::RightCentralDefender,
                    PositionId::LeftCentralDefender,
                    PositionId::LeftBack,
                    PositionId::RightInnerMidfield,
                    PositionId::CentralInnerMidfield,
                    PositionId::LeftInnerMidfield,
                    PositionId::RightForward,
                    PositionId::CentralForward,
                    PositionId::LeftForward,
                ]);
            }
            Formation::F352 => {
                slots.extend_from_slice(&[
                    PositionId::RightCentralDefender,
                    PositionId::MiddleCentralDefender,
                    PositionId::LeftCentralDefender,
                    PositionId::RightWinger,
                    PositionId::RightInnerMidfield,
                    PositionId::CentralInnerMidfield,
                    PositionId::LeftInnerMidfield,
                    PositionId::LeftWinger,
                    PositionId::RightForward,
                    PositionId::LeftForward,
                ]);
            }
            Formation::F343 => {
                slots.extend_from_slice(&[
                    PositionId::RightCentralDefender,
                    PositionId::MiddleCentralDefender,
                    PositionId::LeftCentralDefender,
                    PositionId::RightWinger,
                    PositionId::RightInnerMidfield,
                    PositionId::LeftInnerMidfield,
                    PositionId::LeftWinger,
                    PositionId::RightForward,
                    PositionId::CentralForward,
                    PositionId::LeftForward,
                ]);
            }
            Formation::F253 => {
                slots.extend_from_slice(&[
                    PositionId::RightCentralDefender,
                    PositionId::LeftCentralDefender,
                    PositionId::RightWinger,
                    PositionId::RightInnerMidfield,
                    PositionId::CentralInnerMidfield,
                    PositionId::LeftInnerMidfield,
                    PositionId::LeftWinger,
                    PositionId::RightForward,
                    PositionId::CentralForward,
                    PositionId::LeftForward,
                ]);
            }
        }
        slots
    }
}

#[derive(Debug, Clone)]
pub struct OptimisedLineup {
    pub formation: Formation,
    pub lineup: Lineup,
    pub sector_ratings: HashMap<RatingSector, f64>,
    pub player_ratings: HashMap<PositionId, PositionRating>,
    pub hatstats: f64,
    pub captain: Option<Player>,
    pub set_pieces_taker: Option<Player>,
}

/// Optimiser that finds the best lineup for a formation using Hill Climbing
pub struct LineupOptimiser<'a> {
    model: &'a RatingPredictionModel,
    players: &'a [Player],
}

impl<'a> LineupOptimiser<'a> {
    pub fn new(model: &'a RatingPredictionModel, players: &'a [Player]) -> Self {
        Self { model, players }
    }

    pub fn optimise(&self, formation: Formation) -> OptimisedLineup {
        // ... (existing optimization loop)

        let slots = formation.get_slots();
        let mut best_lineup = self.create_initial_lineup(&slots);
        let mut best_hatstats = self.model.calc_hatstats(&best_lineup, 0);

        // Hill Climbing Configuration
        let max_iterations = 1000;
        let mut no_improvement_count = 0;
        let max_no_improvement = 100;

        for _ in 0..max_iterations {
            let mut improved = false;

            // Try to swap a player in lineup with a bench player
            if let Some(new_lineup) = self.try_swap_bench(&best_lineup, &slots) {
                let new_hatstats = self.model.calc_hatstats(&new_lineup, 0);
                if new_hatstats > best_hatstats {
                    best_lineup = new_lineup;
                    best_hatstats = new_hatstats;
                    improved = true;
                }
            }

            // Try to swap two players within the lineup
            if !improved {
                if let Some(new_lineup) = self.try_swap_roles(&best_lineup) {
                    let new_hatstats = self.model.calc_hatstats(&new_lineup, 0);
                    if new_hatstats > best_hatstats {
                        best_lineup = new_lineup;
                        best_hatstats = new_hatstats;
                        improved = true;
                    }
                }
            }

            // Try changing behaviour of a player
            if !improved {
                if let Some(new_lineup) = self.try_change_behaviour(&best_lineup) {
                    let new_hatstats = self.model.calc_hatstats(&new_lineup, 0);
                    if new_hatstats > best_hatstats {
                        best_lineup = new_lineup;
                        best_hatstats = new_hatstats;
                        improved = true;
                    }
                }
            }

            if improved {
                no_improvement_count = 0;
            } else {
                no_improvement_count += 1;
            }

            if no_improvement_count >= max_no_improvement {
                break;
            }
        }

        // Calculate final ratings
        let mut sector_ratings = HashMap::new();
        for sector in RatingSector::all() {
            sector_ratings.insert(sector, self.model.get_rating(&best_lineup, sector, 0));
        }

        // Calculate individual player ratings
        let mut player_ratings = HashMap::new();
        for pos in &best_lineup.positions {
            let rating = calculate_position_rating(
                self.model,
                &pos.player,
                pos.role_id,
                pos.behaviour,
                &best_lineup,
                0,
            );
            player_ratings.insert(pos.role_id, rating);
        }

        let captain = self.select_captain(&best_lineup);
        let set_pieces_taker = self.select_set_pieces_taker(&best_lineup);

        OptimisedLineup {
            formation,
            lineup: best_lineup,
            sector_ratings,
            player_ratings,
            hatstats: best_hatstats,
            captain,
            set_pieces_taker,
        }
    }

    fn select_captain(&self, lineup: &Lineup) -> Option<Player> {
        let players: Vec<&Player> = lineup.positions.iter().map(|p| &p.player).collect();
        if players.is_empty() {
            return None;
        }

        let mut best_captain = None;
        let mut max_xp_value = -1.0;

        // Sum experience of all players
        let mut total_xp = 0.0;
        for p in &players {
            total_xp += p.Experience as f64;
        }

        for candidate in &players {
            // Add captain's experience again (as per Java logic: (sum(xp) + captain.xp) / 12)
            let current_total_xp = total_xp + candidate.Experience as f64;
            let avg_xp = current_total_xp / 12.0;

            // Penalty for leadership < 7 (Solid)
            // Factor = 1.0 - (7 - leadership) * 0.05
            // Note: Leadership 7 is Solid? In CHPP API, 3 might be standard. Need to verify scale.
            // In CHPP: 1=Disasterous ... 7=Solid, 8=Excellent.
            // Java uses 7 as baseline.
            let leadership = candidate.Leadership as f64;
            let factor = 1.0 - (7.0 - leadership) * 0.05;

            let value = avg_xp * factor;

            if value > max_xp_value {
                max_xp_value = value;
                best_captain = Some((*candidate).clone());
            }
        }

        best_captain
    }

    fn select_set_pieces_taker(&self, lineup: &Lineup) -> Option<Player> {
        // Exclude keeper from set pieces taker candidates (as per Java logic)
        let candidates: Vec<&Player> = lineup
            .positions
            .iter()
            .filter(|p| p.role_id != PositionId::Keeper)
            .map(|p| &p.player)
            .collect();

        if candidates.is_empty() {
            return None;
        }

        let mut best_taker = None;
        let mut max_sp_strength = -1.0;
        let mut best_form = -1;

        for player in candidates {
            let strength = RatingPredictionModel::get_player_set_pieces_strength(player);
            let form = player.PlayerForm as i32;

            if strength > max_sp_strength {
                max_sp_strength = strength;
                best_taker = Some(player.clone());
                best_form = form;
            } else if (strength - max_sp_strength).abs() < 0.001 && form > best_form {
                // Tie breaker on form
                max_sp_strength = strength;
                best_taker = Some(player.clone());
                best_form = form;
            }
        }

        best_taker
    }

    fn create_initial_lineup(&self, slots: &[PositionId]) -> Lineup {
        // Simple greedy initialization: pick best available player for each slot
        // This gives a much better starting point than random
        // Note: This is a simplified greedy approach, real optimal assignment is minimum cost maximum flow or similar
        // But for   Climbing initialization, this is sufficient

        // Use a set to keep track of assigned players
        let mut assigned_players = HashSet::new();
        let mut positions = Vec::new();

        // 1. Assign Keeper
        let keeper_slot = PositionId::Keeper;
        if let Some(keeper) = self.find_best_player_for_slot(keeper_slot, &assigned_players) {
            positions.push(LineupPosition {
                player: keeper.clone(),
                role_id: keeper_slot,
                behaviour: Behaviour::Normal,
                start_minute: 0,
            });
            assigned_players.insert(keeper.PlayerID);
        }

        // 2. Assign other slots
        for &slot in slots {
            if slot == PositionId::Keeper {
                continue;
            } // Already handled

            if let Some(player) = self.find_best_player_for_slot(slot, &assigned_players) {
                positions.push(LineupPosition {
                    player: player.clone(),
                    role_id: slot,
                    behaviour: Behaviour::Normal,
                    start_minute: 0,
                });
                assigned_players.insert(player.PlayerID);
            }
        }

        // If we ran out of players, fill with remaining players (should rarely happen in real game unless squad is tiny)
        // ... (Handling strict squad size constraints is optional for this iteration)

        Lineup {
            positions,
            weather: Weather::default(),
            tactic: TacticType::Normal,
            attitude: Attitude::Normal,
            location: Location::default(),
        }
    }

    fn find_best_player_for_slot(
        &self,
        slot: PositionId,
        assigned: &HashSet<u32>,
    ) -> Option<&Player> {
        // Find best unassigned player for this slot
        // We use evaluate_all_positions logic but simplified/focused on this slot

        let mut best_player = None;
        let mut max_rating = -1.0;

        // Dummy lineup for context (empty)
        let dummy_lineup = Lineup {
            positions: vec![],
            weather: Weather::default(),
            tactic: TacticType::Normal,
            attitude: Attitude::Normal,
            location: Location::default(),
        };

        for player in self.players {
            if assigned.contains(&player.PlayerID) {
                continue;
            }
            if player.InjuryLevel.unwrap_or(0) >= 1 && player.InjuryLevel.unwrap_or(0) < 90 {
                continue;
            } // Skip injured

            // Evaluate player for this specific slot
            // Since evaluate_all_positions is heavy, we can call calculate_position_rating logic directly if exposed,
            // or just use evaluate_all_positions and filter.
            // For now, we reuse evaluate_all_positions as it's available.
            let eval = evaluate_all_positions(self.model, player, &dummy_lineup, 0);

            if let Some(pos_rating) = eval
                .positions
                .iter()
                .find(|p| p.position == slot && p.behaviour == Behaviour::Normal)
            {
                if pos_rating.rating > max_rating {
                    max_rating = pos_rating.rating;
                    best_player = Some(player);
                }
            }
        }

        best_player
    }

    fn try_swap_bench(&self, lineup: &Lineup, _slots: &[PositionId]) -> Option<Lineup> {
        // Randomly pick a slot in lineup
        if lineup.positions.is_empty() {
            return None;
        }
        let idx = fastrand::usize(0..lineup.positions.len());
        let _current_pos = &lineup.positions[idx];

        // Find bench players
        let lineup_ids: HashSet<u32> = lineup.positions.iter().map(|p| p.player.PlayerID).collect();
        let bench_players: Vec<&Player> = self
            .players
            .iter()
            .filter(|p| {
                !lineup_ids.contains(&p.PlayerID)
                    && (p.InjuryLevel.unwrap_or(0) < 1 || p.InjuryLevel.unwrap_or(0) >= 90)
            })
            .collect();

        if bench_players.is_empty() {
            return None;
        }

        let bench_player = bench_players[fastrand::usize(0..bench_players.len())];

        let mut new_lineup = lineup.clone();
        new_lineup.positions[idx].player = bench_player.clone();

        Some(new_lineup)
    }

    fn try_swap_roles(&self, lineup: &Lineup) -> Option<Lineup> {
        if lineup.positions.len() < 2 {
            return None;
        }
        let idx1 = fastrand::usize(0..lineup.positions.len());
        let mut idx2 = fastrand::usize(0..lineup.positions.len());
        while idx1 == idx2 {
            idx2 = fastrand::usize(0..lineup.positions.len());
        }

        let mut new_lineup = lineup.clone();

        // Swap players but KEEP roles/behaviours
        // This simulates: "What if Player A played in Pos B and Player B in Pos A?"
        let p1 = new_lineup.positions[idx1].player.clone();
        let p2 = new_lineup.positions[idx2].player.clone();
        new_lineup.positions[idx1].player = p2;
        new_lineup.positions[idx2].player = p1;

        Some(new_lineup)
    }

    fn try_change_behaviour(&self, lineup: &Lineup) -> Option<Lineup> {
        if lineup.positions.is_empty() {
            return None;
        }
        let idx = fastrand::usize(0..lineup.positions.len());
        let current_pos = &lineup.positions[idx];

        let valid_behaviours = current_pos.role_id.valid_behaviours();
        if valid_behaviours.len() <= 1 {
            return None; // No alternative behaviour
        }

        // Pick a random DIFFERENT behaviour
        let mut new_behaviour = valid_behaviours[fastrand::usize(0..valid_behaviours.len())];
        while new_behaviour == current_pos.behaviour {
            new_behaviour = valid_behaviours[fastrand::usize(0..valid_behaviours.len())];
        }

        let mut new_lineup = lineup.clone();
        new_lineup.positions[idx].behaviour = new_behaviour;

        Some(new_lineup)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chpp::model::{Player, PlayerSkills};
    use crate::rating::model::Team;

    fn create_test_player(id: u32, skills: PlayerSkills) -> Player {
        Player {
            PlayerID: id,
            FirstName: format!("Player {}", id),
            LastName: "Test".to_string(),
            NickName: None,
            PlayerNumber: Some(id),
            Age: 25,
            AgeDays: Some(0),
            TSI: 1000,
            PlayerForm: 7,
            Statement: None,
            Experience: 5,
            Loyalty: 15,
            ReferencePlayerID: None,
            MotherClubBonus: false,
            Leadership: 3,
            Salary: 1000,
            IsAbroad: false,
            Agreeability: 3,
            Aggressiveness: 3,
            Honesty: 3,
            LeagueGoals: None,
            CupGoals: None,
            FriendliesGoals: None,
            CareerGoals: None,
            CareerHattricks: None,
            CareerAssists: None,
            Specialty: None,
            TransferListed: false,
            NationalTeamID: None,
            CountryID: None,
            Caps: None,
            CapsU20: None,
            Cards: None,
            InjuryLevel: None,
            AvatarBlob: None,
            GenderID: None,
            NativeCountryFlag: None,
            Flag: None,
            PlayerSkills: Some(skills),
            ArrivalDate: None,
            PlayerCategoryId: None,
            MotherClub: None,
            NativeCountryID: None,
            NativeLeagueID: None,
            NativeLeagueName: None,
            MatchesCurrentTeam: None,
            GoalsCurrentTeam: None,
            AssistsCurrentTeam: None,
            LastMatch: None,
        }
    }

    #[test]
    fn test_formation_slots() {
        let f442 = Formation::F442;
        let slots = f442.get_slots();
        assert_eq!(slots.len(), 11);
        assert!(slots.contains(&PositionId::Keeper));
        assert!(slots.contains(&PositionId::LeftWinger));
    }

    #[test]
    fn test_optimiser_initialization() {
        let team = Team::default();
        let model = RatingPredictionModel::new(team);
        let players = vec![
            create_test_player(
                1,
                PlayerSkills {
                    KeeperSkill: 20,
                    ..PlayerSkills::default()
                },
            ),
            create_test_player(
                2,
                PlayerSkills {
                    DefenderSkill: 15,
                    ..PlayerSkills::default()
                },
            ),
            // ... need 11 players for full test, but unit test can check partial logic
        ];

        let optimiser = LineupOptimiser::new(&model, &players);
        // Optimization check would require more logic setup
    }
}
