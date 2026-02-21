use crate::chpp::model::Player;
use crate::rating::PlayerSkill;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleStep {
    pub skill: PlayerSkill,
    pub target_level: u8,            // e.g., 15 for Titanic
    pub duration_weeks: Option<u32>, // Optional override
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cycle {
    pub name: String,
    pub description: String,
    pub steps: Vec<CycleStep>,
}

impl Cycle {
    pub fn recommend_trainees<'a>(&self, all_players: &'a [Player]) -> Vec<&'a Player> {
        // Simple recommendation logic:
        // 1. Filter by Age (e.g., < 19)
        // 2. Sort by primary skill of the first step?
        // For now, just age filter.

        all_players
            .iter()
            .filter(|p| p.Age <= 18) // Start young
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyclePlan {
    pub id: Uuid,
    pub cycle: Cycle,
    pub trainee_ids: Vec<u32>,
    #[serde(skip)]
    pub projected_progress: Vec<ProgressPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressPoint {
    pub week: u32,
    pub player_id: u32,
    pub skill: PlayerSkill,
    pub level: f64, // Floating point level (e.g., 8.5)
}

impl Default for CyclePlan {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            cycle: Cycle {
                name: "New Cycle".to_string(),
                description: String::new(),
                steps: Vec::new(),
            },
            trainee_ids: Vec::new(),
            projected_progress: Vec::new(),
        }
    }
}

impl CyclePlan {
    pub fn new_template(
        name: &str,
        description: &str,
        steps: Vec<(PlayerSkill, u8, Option<u32>)>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            cycle: Cycle {
                name: name.to_string(),
                description: description.to_string(),
                steps: steps
                    .into_iter()
                    .map(|(skill, level, duration)| CycleStep {
                        skill,
                        target_level: level,
                        duration_weeks: duration,
                    })
                    .collect(),
            },
            trainee_ids: Vec::new(),
            projected_progress: Vec::new(),
        }
    }
}
