/* match_ratings.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::chpp::error::Error;
use crate::db::schema::match_ratings;
use diesel::prelude::*;

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = match_ratings)]
pub struct NewMatchRating {
    pub match_id: i32,
    pub team_id: i32,
    pub formation: Option<String>,
    pub tactic_type: Option<i32>,
    pub rating_midfield: Option<f64>,
    pub rating_right_def: Option<f64>,
    pub rating_mid_def: Option<f64>,
    pub rating_left_def: Option<f64>,
    pub rating_right_att: Option<f64>,
    pub rating_mid_att: Option<f64>,
    pub rating_left_att: Option<f64>,
}

#[derive(Queryable, Debug, Clone)]
pub struct MatchRating {
    pub match_id: i32,
    pub team_id: i32,
    pub formation: Option<String>,
    pub tactic_type: Option<i32>,
    pub rating_midfield: Option<f64>,
    pub rating_right_def: Option<f64>,
    pub rating_mid_def: Option<f64>,
    pub rating_left_def: Option<f64>,
    pub rating_right_att: Option<f64>,
    pub rating_mid_att: Option<f64>,
    pub rating_left_att: Option<f64>,
}

/// Upsert match ratings for a specific team.
pub fn save_match_ratings(
    conn: &mut SqliteConnection,
    ratings: &[NewMatchRating],
) -> Result<(), Error> {
    diesel::replace_into(match_ratings::table)
        .values(ratings)
        .execute(conn)
        .map_err(|e| Error::Io(format!("Failed to save match ratings: {}", e)))?;
    Ok(())
}

/// Load all stored ratings for a given team ID.
pub fn get_match_ratings(conn: &mut SqliteConnection, tid: u32) -> Result<Vec<MatchRating>, Error> {
    use crate::db::schema::match_ratings::dsl::*;
    match_ratings
        .filter(team_id.eq(tid as i32))
        .load::<MatchRating>(conn)
        .map_err(|e| Error::Io(format!("Failed to load match ratings: {}", e)))
}
