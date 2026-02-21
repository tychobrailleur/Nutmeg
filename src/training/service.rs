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
        let defaults = Self::get_default_templates();

        if !path.exists() {
            log::info!("No training plans found, returning default templates");
            if let Err(e) = Self::save_all(&defaults) {
                log::error!("Failed to save default templates: {}", e);
            }
            return defaults;
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<Vec<CyclePlan>>(&content) {
                Ok(mut plans) => {
                    let mut modified = false;

                    // 1. Deduplicate existing plans by name (keep first occurrence)
                    let mut unique_plans = Vec::new();
                    let mut seen_names = std::collections::HashSet::new();
                    for plan in plans.drain(..) {
                        if seen_names.insert(plan.cycle.name.clone()) {
                            unique_plans.push(plan);
                        } else {
                            modified = true;
                        }
                    }
                    plans = unique_plans;

                    // 2. Merge/Update missing or old defaults
                    for default_plan in defaults {
                        if let Some(existing) = plans
                            .iter_mut()
                            .find(|p| p.cycle.name == default_plan.cycle.name)
                        {
                            // If it's a default but has 1-week placeholders, update it with expert durations
                            let has_placeholders =
                                existing.cycle.steps.iter().any(|s| {
                                    s.duration_weeks.is_none() || s.duration_weeks == Some(1)
                                });

                            if has_placeholders {
                                existing.cycle.steps = default_plan.cycle.steps.clone();
                                existing.cycle.description = default_plan.cycle.description.clone();
                                modified = true;
                            }
                        } else {
                            plans.push(default_plan);
                            modified = true;
                        }
                    }

                    if modified {
                        if let Err(e) = Self::save_all(&plans) {
                            log::error!("Failed to save cleaned/merged training plans: {}", e);
                        }
                    }
                    plans
                }
                Err(e) => {
                    log::error!("Failed to deserialize training plans: {}", e);
                    defaults
                }
            },
            Err(e) => {
                log::error!("Failed to read training plans file: {}", e);
                defaults
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
        vec![
            // 1. The Modern Midfielder (Long Cycle)
            CyclePlan::new_template(
                "Modern Midfielder (Long Cycle)",
                "A comprehensive cycle to create a world-class inner midfielder. Optimises playmaking, passing, and defending over several seasons.",
                vec![
                    (PlayerSkill::Playmaking, 15, Some(45)),
                    (PlayerSkill::Passing, 11, Some(24)),
                    (PlayerSkill::Defending, 10, Some(20)),
                    (PlayerSkill::Playmaking, 17, Some(35)),
                ],
            ),
            // 2. The Quick Profit (Short Cycle)
            CyclePlan::new_template(
                "Quick Profit (Short Cycle)",
                "A short 2-season cycle designed to train young strikers for a quick turnaround and profit on the transfer market.",
                vec![
                    (PlayerSkill::Scoring, 10, Some(28)),
                    (PlayerSkill::Passing, 8, Some(14)),
                ],
            ),
            // 3. The Solid Defender
            CyclePlan::new_template(
                "Solid Defender",
                "Focuses on building a high-level defender with strong secondary skills in playmaking and passing for better team contribution.",
                vec![
                    (PlayerSkill::Defending, 14, Some(40)),
                    (PlayerSkill::Playmaking, 9, Some(18)),
                    (PlayerSkill::Passing, 8, Some(12)),
                ],
            ),
            // 4. The Extreme Winger
            CyclePlan::new_template(
                "Extreme Winger",
                "Dedicated winger training aimed at maximising side attack ratings. Includes significant playmaking and passing for versatility.",
                vec![
                    (PlayerSkill::Winger, 15, Some(38)),
                    (PlayerSkill::Playmaking, 11, Some(22)),
                    (PlayerSkill::Passing, 10, Some(16)),
                ],
            ),
            // 5. The Creative Forward
            CyclePlan::new_template(
                "Creative Forward",
                "Develops a hybrid player excellent for technical forward roles or creative play tactics, balancing scoring and passing.",
                vec![
                    (PlayerSkill::Scoring, 13, Some(32)),
                    (PlayerSkill::Passing, 12, Some(24)),
                    (PlayerSkill::Playmaking, 9, Some(15)),
                ],
            ),
        ]
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
        Player {
            PlayerID: 1,
            Age: 17,
            PlayerSkills: Some(PlayerSkills {
                KeeperSkill: 1,
                DefenderSkill: 5,
                PlaymakerSkill: 5,
                WingerSkill: 3,
                PassingSkill: 4,
                ScorerSkill: 3,
                SetPiecesSkill: 3,
                StaminaSkill: 3,
            }),
            ..Default::default()
        }
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

        let plan = CyclePlan {
            cycle,
            trainee_ids: vec![player.PlayerID],
            ..Default::default()
        };

        let progress = TrainingService::calculate_progress(&plan, &[player]);

        assert!(!progress.is_empty());

        let last_point = progress.last().unwrap();
        assert!(last_point.level >= 7.0);
        assert!(last_point.week > 0);

        println!("Weeks needed: {}", last_point.week);
    }
}
