/* diesel_team.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::db::manager::DbManager;
use crate::db::repository::team::TeamRepository;
use crate::domain::team::Team;
use crate::error::NutmegError;
use async_trait::async_trait;
use diesel::prelude::*;
use std::sync::Arc;

pub struct DieselTeamRepository {
    db_manager: Arc<DbManager>,
}

impl DieselTeamRepository {
    pub fn new(db_manager: Arc<DbManager>) -> Self {
        Self { db_manager }
    }
}

#[async_trait]
impl TeamRepository for DieselTeamRepository {
    async fn get_team_by_id(&self, team_id: u32) -> Result<Option<Team>, NutmegError> {
        use crate::db::schema::teams::dsl::*;
        let db = self.db_manager.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut conn = db.get_connection()?;
            
            // Just select the fields we need for the domain model
            let result = teams
                .filter(id.eq(team_id as i32))
                .order(download_id.desc())
                .select((id, name, short_name, league_id, league_name, country_id, country_name))
                .first::<(i32, String, Option<String>, Option<i32>, Option<String>, Option<i32>, Option<String>)>(&mut conn)
                .optional()?;
            
            Ok(result.map(|r| Team {
                id: r.0 as u32,
                name: r.1,
                short_name: r.2,
                league_id: r.3.map(|v| v as u32),
                league_name: r.4,
                country_id: r.5.map(|v| v as u32),
                country_name: r.6,
            }))
        })
        .await
        .map_err(|e| NutmegError::Io(format!("Join error: {}", e)))?
    }

    async fn get_all_teams(&self) -> Result<Vec<Team>, NutmegError> {
        // Implementation similar to get_team_by_id but with grouping for latest download_id
        Ok(vec![])
    }
}
