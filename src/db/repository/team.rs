/* team.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::domain::team::Team;
use crate::error::NutmegError;
use async_trait::async_trait;

#[async_trait]
pub trait TeamRepository: Send + Sync {
    async fn get_team_by_id(&self, team_id: u32) -> Result<Option<Team>, NutmegError>;
    async fn get_all_teams(&self) -> Result<Vec<Team>, NutmegError>;
    // Other methods as needed...
}
