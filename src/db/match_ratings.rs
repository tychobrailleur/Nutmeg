/* match_ratings.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::chpp::error::Error;
use crate::db::schema::match_ratings;
use diesel::prelude::*;

#[derive(Insertable)]
#[diesel(table_name = match_ratings)]
pub struct NewMatchRating {
    pub match_id: i32,
    pub team_id: i32,
    pub download_id: i32,
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
#[diesel(table_name = match_ratings)]
pub struct MatchRating {
    pub match_id: i32,
    pub team_id: i32,
    pub download_id: i32,
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

/// Insert match ratings. Each (match_id, team_id, download_id) is unique; no
/// conflicts should ever occur because ratings are written exactly once per
/// match per download.
pub fn save_match_ratings(
    conn: &mut SqliteConnection,
    ratings: &[NewMatchRating],
) -> Result<(), Error> {
    diesel::insert_into(match_ratings::table)
        .values(ratings)
        .execute(conn)
        .map_err(|e| Error::Io(format!("Failed to save match ratings: {}", e)))?;
    Ok(())
}

/// Load the most recent ratings for each (match_id, team_id) pair for a given
/// team. Because the table is insert-only, multiple rows per (match_id, team_id)
/// may exist; we keep the one with the highest download_id.
pub fn get_match_ratings(conn: &mut SqliteConnection, tid: u32) -> Result<Vec<MatchRating>, Error> {
    use crate::db::schema::match_ratings::dsl::*;

    // Fetch all rows for the team ordered so the latest download_id comes first.
    let rows: Vec<MatchRating> = match_ratings
        .filter(team_id.eq(tid as i32))
        .order((match_id.asc(), download_id.desc()))
        .load::<MatchRating>(conn)
        .map_err(|e| Error::Io(format!("Failed to load match ratings: {}", e)))?;

    // Deduplicate: keep the first (= highest download_id) row per (match_id, team_id).
    let mut seen = std::collections::HashSet::new();
    let deduped = rows
        .into_iter()
        .filter(|r| seen.insert((r.match_id, r.team_id)))
        .collect();

    Ok(deduped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::download_entries::create_download;
    use crate::db::manager::DbManager;
    use serial_test::serial;

    fn make_rating(match_id: i32, team_id: i32, download_id: i32, midfield: f64) -> NewMatchRating {
        NewMatchRating {
            match_id,
            team_id,
            download_id,
            formation: None,
            tactic_type: None,
            rating_midfield: Some(midfield),
            rating_right_def: None,
            rating_mid_def: None,
            rating_left_def: None,
            rating_right_att: None,
            rating_mid_att: None,
            rating_left_att: None,
        }
    }

    fn create_dl(conn: &mut SqliteConnection) -> i32 {
        create_download(conn, "2026-01-01T00:00:00Z", "completed")
            .expect("Failed to create download")
    }

    #[test]
    #[serial]
    fn test_save_and_load_match_ratings() {
        let db = DbManager::from_url(":memory:");
        db.run_migrations().expect("migrations");
        let mut conn = db.get_connection().expect("conn");
        let dl = create_dl(&mut conn);

        let ratings = vec![
            make_rating(1001, 10, dl, 7.5),
            make_rating(1002, 10, dl, 6.0),
        ];
        save_match_ratings(&mut conn, &ratings).expect("save");

        let loaded = get_match_ratings(&mut conn, 10).expect("load");
        assert_eq!(loaded.len(), 2);
    }

    /// Verifies the insert-only invariant: each (match_id, team_id, download_id)
    /// must be unique. Inserting the same triple twice must fail.
    #[test]
    #[serial]
    fn test_duplicate_insert_fails() {
        let db = DbManager::from_url(":memory:");
        db.run_migrations().expect("migrations");
        let mut conn = db.get_connection().expect("conn");
        let dl = create_dl(&mut conn);

        let r = make_rating(1001, 10, dl, 7.5);
        save_match_ratings(&mut conn, &[r]).expect("first insert ok");

        let r2 = make_rating(1001, 10, dl, 8.0);
        let result = save_match_ratings(&mut conn, &[r2]);
        assert!(
            result.is_err(),
            "Second insert with same (match_id, team_id, download_id) must fail — \
             ON CONFLICT clauses are forbidden by the insert-only pattern"
        );
    }

    /// A second download of the same match produces a second row; get_match_ratings
    /// returns only the latest.
    #[test]
    #[serial]
    fn test_get_returns_latest_download() {
        let db = DbManager::from_url(":memory:");
        db.run_migrations().expect("migrations");
        let mut conn = db.get_connection().expect("conn");
        let dl1 = create_dl(&mut conn);
        let dl2 = create_dl(&mut conn);

        save_match_ratings(&mut conn, &[make_rating(1001, 10, dl1, 6.0)]).expect("save dl1");
        save_match_ratings(&mut conn, &[make_rating(1001, 10, dl2, 8.0)]).expect("save dl2");

        let loaded = get_match_ratings(&mut conn, 10).expect("load");
        assert_eq!(
            loaded.len(),
            1,
            "deduplication should return one row per match"
        );
        assert_eq!(
            loaded[0].rating_midfield,
            Some(8.0),
            "should return the latest download's value"
        );
    }
}
