use crate::chpp::model::Player;
use crate::rating::model::Team;
use crate::rating::{
    optimiser::{Formation, LineupOptimiser, OptimisedLineup},
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
            let opt_lineup = optimiser.optimise(formation);
            results.push(opt_lineup);
        }
        results
    }
}
