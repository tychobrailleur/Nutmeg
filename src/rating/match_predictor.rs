/* match_predictor.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use super::types::{RatingSector, TacticType};
use std::collections::HashMap;

/// Result of a match prediction
#[derive(Debug, Clone, Default)]
pub struct PredictionResult {
    pub win_prob: f64,
    pub draw_prob: f64,
    pub loss_prob: f64,
    pub expected_goals_for: f64,
    pub expected_goals_against: f64,
}

/// Predicts match outcomes based on sector ratings and tactics
pub struct MatchPredictor;

impl MatchPredictor {
    /// Predicts the outcome of a match between two teams.
    pub fn predict(
        our_ratings: &HashMap<RatingSector, f64>,
        opp_ratings: &HashMap<RatingSector, f64>,
        our_tactic: TacticType,
        opp_tactic: TacticType,
    ) -> PredictionResult {
        // 1. Calculate Midfield Possession
        let our_mid = *our_ratings.get(&RatingSector::Midfield).unwrap_or(&0.75);
        let opp_mid = *opp_ratings.get(&RatingSector::Midfield).unwrap_or(&0.75);

        let possession = our_mid / (our_mid + opp_mid);

        // 2. Distributions of regular chances (5 exclusive for A, 5 exclusive for B, 5 shared)
        // Expected chances for us:
        // Exclusive: 5 * possession
        // Shared: 5 * possession
        // If we fail the shared check, opponent gets it.
        // We simplified here to the standard community-accepted expected chances:
        let expected_chances_us = 5.0 * possession + 5.0 * possession; // Simplified linear model for now
        let expected_chances_them = 5.0 * (1.0 - possession) + 5.0 * (1.0 - possession);

        // 3. Goal Conversion using the Power of 3 formula
        // Sectors: Left (25%), Centre (50%), Right (25%)
        // Apply tactic redirection (AIM / AOW)
        let (mut weight_left, mut weight_central, mut weight_right) = (0.25, 0.50, 0.25);
        if our_tactic == TacticType::AttackOnWings {
            let redirection = 0.15; // Approx 15% redirected to wings
            weight_central -= redirection;
            weight_left += redirection / 2.0;
            weight_right += redirection / 2.0;
        } else if our_tactic == TacticType::AttackInTheMiddle {
            let redirection = 0.15; // Approx 15% redirected to middle
            weight_left -= redirection / 2.0;
            weight_right -= redirection / 2.0;
            weight_central += redirection;
        }

        let sectors = [
            (
                RatingSector::AttackLeft,
                RatingSector::DefenceRight,
                weight_left,
            ),
            (
                RatingSector::AttackCentral,
                RatingSector::DefenceCentral,
                weight_central,
            ),
            (
                RatingSector::AttackRight,
                RatingSector::DefenceLeft,
                weight_right,
            ),
        ];

        let mut prob_scoring_for = 0.0;
        for (att_sector, def_sector, weight) in sectors.iter() {
            let att = *our_ratings.get(att_sector).unwrap_or(&0.75);
            let def = *opp_ratings.get(def_sector).unwrap_or(&0.75);

            let p_goal = if att + def > 0.0 {
                att.powi(3) / (att.powi(3) + def.powi(3))
            } else {
                0.0
            };
            prob_scoring_for += p_goal * weight;
        }

        // Opponent tactics
        let (mut o_weight_left, mut o_weight_central, mut o_weight_right) = (0.25, 0.50, 0.25);
        if opp_tactic == TacticType::AttackOnWings {
            let redirection = 0.15;
            o_weight_central -= redirection;
            o_weight_left += redirection / 2.0;
            o_weight_right += redirection / 2.0;
        } else if opp_tactic == TacticType::AttackInTheMiddle {
            let redirection = 0.15;
            o_weight_left -= redirection / 2.0;
            o_weight_right -= redirection / 2.0;
            o_weight_central += redirection;
        }

        let sectors_opp = [
            (
                RatingSector::AttackLeft,
                RatingSector::DefenceRight,
                o_weight_left,
            ),
            (
                RatingSector::AttackCentral,
                RatingSector::DefenceCentral,
                o_weight_central,
            ),
            (
                RatingSector::AttackRight,
                RatingSector::DefenceLeft,
                o_weight_right,
            ),
        ];

        let mut prob_scoring_against = 0.0;
        for (att_sector, def_sector, weight) in sectors_opp.iter() {
            let att = *opp_ratings.get(att_sector).unwrap_or(&0.75);
            let def = *our_ratings.get(def_sector).unwrap_or(&0.75);

            let p_goal = if att + def > 0.0 {
                att.powi(3) / (att.powi(3) + def.powi(3))
            } else {
                0.0
            };
            prob_scoring_against += p_goal * weight;
        }

        let mut xg_for = expected_chances_us * prob_scoring_for;
        let mut xg_against = expected_chances_them * prob_scoring_against;

        // Counter-Attack logic (Simplified)
        // If opponent possession is high but they fail to score, we get CA chances.
        if our_tactic == TacticType::CounterAttacks && possession < 0.45 {
            // Very simplified: adds ~1.5 extra chances if possession is low
            let extra_chances = (0.5 - possession) * 5.0;
            xg_for += extra_chances * prob_scoring_for;
        }
        if opp_tactic == TacticType::CounterAttacks && possession > 0.55 {
            let extra_chances = (possession - 0.5) * 5.0;
            xg_against += extra_chances * prob_scoring_against;
        }

        // Poisson distribution approximation for Win/Draw/Loss
        // For a more accurate result, we'd iterate over scoring possibilities (e.g. up to 10 goals)
        Self::calculate_wdl_poisson(xg_for, xg_against)
    }

    fn calculate_wdl_poisson(lambda_for: f64, lambda_against: f64) -> PredictionResult {
        let mut win = 0.0;
        let mut draw = 0.0;
        let mut loss = 0.0;

        // Calculate probabilities for up to 10 goals for each side
        for i in 0..11 {
            let p_i = Self::poisson(lambda_for, i);
            for j in 0..11 {
                let p_j = Self::poisson(lambda_against, j);
                let p_ij = p_i * p_j;

                if i > j {
                    win += p_ij;
                } else if i == j {
                    draw += p_ij;
                } else {
                    loss += p_ij;
                }
            }
        }

        PredictionResult {
            win_prob: win,
            draw_prob: draw,
            loss_prob: loss,
            expected_goals_for: lambda_for,
            expected_goals_against: lambda_against,
        }
    }

    fn poisson(lambda: f64, k: u32) -> f64 {
        let e_neg_lambda = (-lambda).exp();
        let mut k_fact = 1.0;
        for i in 1..=k {
            k_fact *= i as f64;
        }
        (lambda.powi(k as i32) * e_neg_lambda) / k_fact
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poisson() {
        // Lambda 1.0, 0 goals should be ~36.8%
        let p0 = MatchPredictor::poisson(1.0, 0);
        assert!((p0 - 0.3678).abs() < 0.001);
    }

    #[test]
    fn test_prediction_even() {
        let mut ratings = HashMap::new();
        for sector in RatingSector::all() {
            ratings.insert(sector, 10.0);
        }

        let res =
            MatchPredictor::predict(&ratings, &ratings, TacticType::Normal, TacticType::Normal);

        // Even ratings should result in even win/loss probabilities
        assert!((res.win_prob - res.loss_prob).abs() < 0.01);
        assert!((res.expected_goals_for - res.expected_goals_against).abs() < 0.01);
    }
}
