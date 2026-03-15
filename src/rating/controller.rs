use crate::chpp::model::Player;
use crate::rating::model::Team;
use crate::rating::{
    optimiser::{Formation, LineupOptimiser, OptimisationGoal, OptimisedLineup},
    RatingPredictionModel,
};

pub struct RatingController;

impl RatingController {
    pub fn calculate_best_lineups(players: &[Player]) -> Vec<OptimisedLineup> {
        let team = Team::default();
        let model = RatingPredictionModel::new(team);
        let optimiser = LineupOptimiser::new(&model, players);

        let mut results = Vec::new();
        for formation in Formation::all() {
            let opt_lineup = optimiser.optimise(formation, OptimisationGoal::MaxHatstats);
            results.push(opt_lineup);
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::manager::DbManager;
    use crate::db::teams::get_players_for_team;
    use crate::rating::types::RatingSector;

    #[test]
    fn test_lineup_logic() {
        let db = DbManager::new();
        let mut conn = db.get_connection().unwrap();
        let team_id = 280747;
        let players = get_players_for_team(&mut conn, team_id).unwrap();
        println!("Loaded {} players for team {}", players.len(), team_id);

        let lineups = RatingController::calculate_best_lineups(&players);
        println!("Calculated {} lineups.", lineups.len());

        let mid_avg = 5.0;
        let rd_avg = 5.0;
        let cd_avg = 5.0;
        let ld_avg = 5.0;
        let ra_avg = 5.0;
        let ca_avg = 5.0;
        let la_avg = 5.0;

        let mut best_lineup_name = String::new();
        let mut best_score = -9999.0;

        for lineup in lineups {
            let m = lineup
                .sector_ratings
                .get(&RatingSector::Midfield)
                .unwrap_or(&0.0)
                - mid_avg;
            let al = lineup
                .sector_ratings
                .get(&RatingSector::AttackLeft)
                .unwrap_or(&0.0)
                - rd_avg;
            let ac = lineup
                .sector_ratings
                .get(&RatingSector::AttackCentral)
                .unwrap_or(&0.0)
                - cd_avg;
            let ar = lineup
                .sector_ratings
                .get(&RatingSector::AttackRight)
                .unwrap_or(&0.0)
                - ld_avg;
            let dl = lineup
                .sector_ratings
                .get(&RatingSector::DefenceLeft)
                .unwrap_or(&0.0)
                - ra_avg;
            let dc = lineup
                .sector_ratings
                .get(&RatingSector::DefenceCentral)
                .unwrap_or(&0.0)
                - ca_avg;
            let dr = lineup
                .sector_ratings
                .get(&RatingSector::DefenceRight)
                .unwrap_or(&0.0)
                - la_avg;

            let score = (m * 3.0) + al + ac + ar + dl + dc + dr;

            if score > best_score {
                best_score = score;
                best_lineup_name = lineup.formation.name().to_string();
            }
            println!("Formation: {:?}, Score: {}", lineup.formation.name(), score);
        }

        println!("Best score: {}, name: {}", best_score, best_lineup_name);
        assert!(false); // Force panic to show stdout
    }
}
