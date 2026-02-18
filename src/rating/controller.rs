use crate::chpp::model::Player;
use crate::rating::{RatingPredictionModel, optimizer::{LineupOptimizer, OptimizedLineup, Formation}};
use crate::rating::model::Team;

pub struct RatingController;

impl RatingController {
    pub fn calculate_best_lineups(players: &[Player]) -> Vec<OptimizedLineup> {
        let team = Team::default(); 
        let model = RatingPredictionModel::new(team);
        let optimizer = LineupOptimizer::new(&model, players);

        let mut results = Vec::new();
        for formation in Formation::all() {
            let opt_lineup = optimizer.optimize(formation);
            results.push(opt_lineup);
        }
        results
    }
}
