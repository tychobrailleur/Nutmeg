/* context.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::chpp::model::{Country, Currency, Language, Player, Region, Team, User};
use crate::db::manager::DbManager;
use crate::db::teams::{
    get_latest_country, get_latest_region, get_latest_user, get_team, get_user_id_for_team,
};
use log::{error, info};
use std::sync::Arc;

#[derive(Default, Debug, Clone)]
pub struct AppContext {
    pub team: Option<Team>,
    pub user: Option<User>,
    pub country: Option<Country>,
    pub region: Option<Region>,
    pub currency: Option<Currency>,
    pub language: Option<Language>,
    pub player: Option<Player>,
}

#[derive(Clone)]
pub struct ContextService {
    db_manager: Arc<DbManager>,
}

impl ContextService {
    pub fn new(db_manager: Arc<DbManager>) -> Self {
        Self { db_manager }
    }

    /// Loads the context information associated with a selected team.
    pub fn load_team_context(&self, team_id: u32) -> AppContext {
        let mut ctx = AppContext::default();

        if let Ok(mut conn) = self.db_manager.get_connection() {
            match get_team(&mut conn, team_id) {
                Ok(Some(team)) => {
                    ctx.team = Some(team.clone());

                    // Get Country
                    if let Some(c) = &team.Country {
                        let cid = c.CountryID;
                        if let Ok(Some(country)) = get_latest_country(&mut conn, cid as i32) {
                            ctx.country = Some(country);
                        }
                    }

                    // Get Region
                    if let Some(r) = &team.Region {
                        if let Ok(Some(region)) = get_latest_region(&mut conn, r.RegionID as i32) {
                            ctx.region = Some(region);
                        }
                    }

                    // Get User (and Language)
                    if let Ok(Some(uid)) = get_user_id_for_team(&mut conn, team_id as i32) {
                        if let Ok(Some(user)) = get_latest_user(&mut conn, uid) {
                            ctx.language = Some(user.Language.clone());
                            ctx.user = Some(user);
                        }
                    }

                    // Populate Currency from Country if available
                    if let Some(c) = &ctx.country {
                        ctx.currency = c.Currency.clone();
                    }

                    info!(
                        "Context updated: Team={}, Country={:?}, User={:?}",
                        team.TeamName,
                        ctx.country.as_ref().map(|c| c.CountryName.as_str()),
                        ctx.user.as_ref().map(|u| u.Loginname.as_str())
                    );
                }
                Ok(None) => {
                    error!("Team {} not found in DB", team_id);
                }
                Err(e) => {
                    error!("Failed to fetch team details: {}", e);
                }
            }
        } else {
            error!("Failed to get DB connection for context loading");
        }

        ctx
    }

    pub fn update_current_player(&self, ctx: &mut AppContext, player: Player) {
        info!("Context updated: Player={}", player.LastName);
        ctx.player = Some(player);
    }

    pub fn clear_current_player(&self, ctx: &mut AppContext) {
        ctx.player = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chpp::model::{Country, Currency, Language, SupporterTier, Team, User};
    use crate::db::teams::{save_team, DownloadEntity};
    use diesel::prelude::*;

    #[test]
    fn test_load_team_context() {
        let db_manager = Arc::new(DbManager::from_url("file::memory:?cache=shared"));
        db_manager.run_migrations().expect("Migrations failed");

        let mut conn = db_manager.get_connection().expect("Connection failed");

        // Seed Download
        let d1 = DownloadEntity {
            id: 1,
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            status: "completed".to_string(),
        };
        diesel::insert_into(crate::db::schema::downloads::table)
            .values(&d1)
            .execute(&mut conn)
            .expect("Failed to create download");

        // Seed Data
        let user = User {
            UserID: 1,
            Name: "TestUser".to_string(),
            Loginname: "testuser".to_string(),
            SupporterTier: SupporterTier::None,
            SignupDate: "".to_string(),
            ActivationDate: "".to_string(),
            LastLoginDate: "".to_string(),
            HasManagerLicense: false,
            Language: Language {
                LanguageID: 2,
                LanguageName: "English".to_string(),
            },
        };

        let currency = Currency {
            CurrencyID: 10,
            CurrencyName: "Dollar".to_string(),
            Rate: Some(1.0),
            Symbol: Some("$".to_string()),
        };

        let country = Country {
            CountryID: 5,
            CountryName: "USA".to_string(),
            Currency: Some(currency),
            CountryCode: None,
            DateFormat: None,
            TimeFormat: None,
        };

        let mut team = Team::default();
        team.TeamID = "100".to_string();
        team.TeamName = "Test Team".to_string();
        team.Country = Some(country);

        save_team(&mut conn, &team, &user, 1).expect("Saved team");

        // Test Service
        let service = ContextService::new(db_manager.clone());
        let ctx = service.load_team_context(100);

        assert!(ctx.team.is_some());
        assert_eq!(ctx.team.unwrap().TeamName, "Test Team");

        assert!(ctx.user.is_some());
        assert_eq!(ctx.user.unwrap().Loginname, "testuser");

        assert!(ctx.country.is_some());
        assert_eq!(ctx.country.unwrap().CountryName, "USA");

        assert!(ctx.currency.is_some());
        assert_eq!(ctx.currency.unwrap().CurrencyName, "Dollar");

        assert!(ctx.language.is_some());
        assert_eq!(ctx.language.unwrap().LanguageName, "English");
    }
}
