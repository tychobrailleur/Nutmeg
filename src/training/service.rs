use super::calculator::calculate_training_progress;
use super::cycle::{CyclePlan, ProgressPoint};
use crate::chpp::model::Player;
use crate::rating::PlayerSkill;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct TrainingService;

impl TrainingService {
    fn get_storage_path() -> PathBuf {
        let home_dir = std::env::var("HOME").expect("HOME not set");
        Path::new(&home_dir)
            .join(".nutmeg")
            .join("training_plans.json")
    }

    pub fn load_plans() -> Vec<CyclePlan> {
        let path = Self::get_storage_path();
        if !path.exists() {
            log::info!("No training plans found, returning default templates");
            let defaults = Self::get_default_templates();
            if let Err(e) = Self::save_all(&defaults) {
                log::error!("Failed to save default templates: {}", e);
            }
            return defaults;
        }

        match std::fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<Vec<CyclePlan>>(&content) {
                Ok(plans) => {
                    if plans.is_empty() {
                        log::info!("Training plans file empty, returning default templates");
                        let defaults = Self::get_default_templates();
                        if let Err(e) = Self::save_all(&defaults) {
                            log::error!("Failed to save default templates: {}", e);
                        }
                        defaults
                    } else {
                        plans
                    }
                }
                Err(e) => {
                    log::error!("Failed to deserialize training plans: {}", e);
                    Self::get_default_templates()
                }
            },
            Err(e) => {
                log::error!("Failed to read training plans file: {}", e);
                Self::get_default_templates()
            }
        }
    }

    pub fn save_plan(plan: &CyclePlan) -> Result<(), Box<dyn std::error::Error>> {
        let mut plans = Self::load_plans();

        // Update existing or add new
        if let Some(idx) = plans.iter().position(|p| p.id == plan.id) {
            plans[idx] = plan.clone();
        } else {
            plans.push(plan.clone());
        }

        Self::save_all(&plans)
    }

    pub fn delete_plan(plan_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let mut plans = Self::load_plans();
        plans.retain(|p| p.id != plan_id);
        Self::save_all(&plans)
    }

    fn save_all(plans: &[CyclePlan]) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_storage_path();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(plans)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn get_default_templates() -> Vec<CyclePlan> {
        let mut plans = Vec::new();

        // 1. Goalkeeper Cycle
        plans.push(CyclePlan::new_template(
            "Goalkeeper Cycle",
            "Standard cycle for GK training",
            vec![
                (PlayerSkill::Keeper, 10, None),
                (PlayerSkill::Defending, 8, None),
                (PlayerSkill::SetPieces, 10, None),
            ],
        ));

        // 2. Defender Cycle -> Playmaking
        plans.push(CyclePlan::new_template(
            "Defender Cycle",
            "Defending then Playmaking",
            vec![
                (PlayerSkill::Defending, 12, None),
                (PlayerSkill::Playmaking, 8, None),
                (PlayerSkill::Passing, 6, None),
            ],
        ));

        // 3. Playmaker Cycle
        plans.push(CyclePlan::new_template(
            "Playmaker Cycle",
            "PM -> Passing -> Defending",
            vec![
                (PlayerSkill::Playmaking, 13, None),
                (PlayerSkill::Passing, 8, None),
                (PlayerSkill::Defending, 7, None),
            ],
        ));

        // 4. Winger Cycle
        plans.push(CyclePlan::new_template(
            "Winger Cycle",
            "Winger -> Playmaking -> Passing",
            vec![
                (PlayerSkill::Winger, 12, None),
                (PlayerSkill::Playmaking, 9, None),
                (PlayerSkill::Passing, 7, None),
                (PlayerSkill::Defending, 6, None),
            ],
        ));

        // 5. Striker Cycle
        plans.push(CyclePlan::new_template(
            "Striker Cycle",
            "Scoring -> Passing -> Winger",
            vec![
                (PlayerSkill::Scoring, 13, None),
                (PlayerSkill::Passing, 8, None),
                (PlayerSkill::Winger, 7, None),
            ],
        ));

        plans
    }

    pub fn calculate_progress(plan: &CyclePlan, trainees: &[Player]) -> Vec<ProgressPoint> {
        let mut progress_points = Vec::new();
        let cycle = &plan.cycle;

        for trainee in trainees {
            let mut current_age = trainee.Age;
            let mut current_week = 0;

            for step in &cycle.steps {
                let mut current_skill_level = if let Some(skills) = &trainee.PlayerSkills {
                    match step.skill {
                        PlayerSkill::Keeper => skills.KeeperSkill as f64,
                        PlayerSkill::Defending => skills.DefenderSkill as f64,
                        PlayerSkill::Playmaking => skills.PlaymakerSkill as f64,
                        PlayerSkill::Winger => skills.WingerSkill as f64,
                        PlayerSkill::Passing => skills.PassingSkill as f64,
                        PlayerSkill::Scoring => skills.ScorerSkill as f64,
                        PlayerSkill::SetPieces => skills.SetPiecesSkill as f64,
                        _ => 3.0,
                    }
                } else {
                    3.0
                };

                while current_skill_level < step.target_level as f64 {
                    current_week += 1;

                    let gain = calculate_training_progress(
                        current_skill_level,
                        step.skill,
                        current_age as u8,
                        1.0,
                        0.15,
                    );

                    current_skill_level += gain;

                    if current_week % 16 == 0 {
                        current_age += 1;
                    }

                    progress_points.push(ProgressPoint {
                        week: current_week,
                        player_id: trainee.PlayerID,
                        skill: step.skill,
                        level: current_skill_level,
                    });
                }
            }
        }
        progress_points
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chpp::model::{Player, PlayerSkills};
    use crate::rating::PlayerSkill;
    use crate::training::cycle::{Cycle, CyclePlan, CycleStep};

    fn create_test_player() -> Player {
        let mut p = Player::default();
        p.PlayerID = 1;
        p.Age = 17;
        p.PlayerSkills = Some(PlayerSkills {
            KeeperSkill: 1,
            DefenderSkill: 5,
            PlaymakerSkill: 5,
            WingerSkill: 3,
            PassingSkill: 4,
            ScorerSkill: 3,
            SetPiecesSkill: 3,
            StaminaSkill: 3,
        });
        p
    }

    #[test]
    fn test_calculate_progress() {
        let player = create_test_player();
        let cycle = Cycle {
            name: "Test Cycle".to_string(),
            description: "A test cycle".to_string(),
            steps: vec![CycleStep {
                skill: PlayerSkill::Playmaking,
                target_level: 7, // 5 -> 7
                duration_weeks: None,
            }],
        };

        let mut plan = CyclePlan::default();
        plan.cycle = cycle;
        plan.trainee_ids = vec![player.PlayerID];

        let progress = TrainingService::calculate_progress(&plan, &vec![player]);

        assert!(!progress.is_empty());

        let last_point = progress.last().unwrap();
        assert!(last_point.level >= 7.0);
        assert!(last_point.week > 0);

        println!("Weeks needed: {}", last_point.week);
    }
}
