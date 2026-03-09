/* series.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::chpp::error::Error;
use crate::chpp::model::{LeagueDetailsData, MatchesData};
use crate::db::schema::{league_unit_teams, league_units, matches};

#[derive(Queryable, Identifiable, Debug)]
#[diesel(table_name = league_units)]
#[diesel(primary_key(unit_id, download_id))]
pub struct LeagueUnit {
    pub unit_id: i32,
    pub download_id: i32,
    pub unit_name: String,
    pub league_level: i32,
    pub max_number_of_teams: Option<i32>,
    pub current_match_round: Option<i32>,
}

#[derive(Insertable)]
#[diesel(table_name = league_units)]
pub struct NewLeagueUnit<'a> {
    pub unit_id: i32,
    pub download_id: i32,
    pub unit_name: &'a str,
    pub league_level: i32,
    pub max_number_of_teams: Option<i32>,
    pub current_match_round: Option<i32>,
}

#[derive(Queryable, Identifiable, Debug)]
#[diesel(table_name = league_unit_teams)]
#[diesel(primary_key(unit_id, team_id, download_id))]
pub struct LeagueUnitTeam {
    pub unit_id: i32,
    pub team_id: i32,
    pub download_id: i32,
    pub team_name: String,
    pub position: i32,
    pub points: i32,
    pub matches_played: i32,
    pub goals_for: i32,
    pub goals_against: i32,
    pub won: i32,
    pub draws: i32,
    pub lost: i32,
}

#[derive(Insertable)]
#[diesel(table_name = league_unit_teams)]
pub struct NewLeagueUnitTeam<'a> {
    pub unit_id: i32,
    pub team_id: i32,
    pub download_id: i32,
    pub team_name: &'a str,
    pub position: i32,
    pub points: i32,
    pub matches_played: i32,
    pub goals_for: i32,
    pub goals_against: i32,
    pub won: i32,
    pub draws: i32,
    pub lost: i32,
}

#[derive(Queryable, Identifiable, Debug)]
#[diesel(table_name = matches)]
#[diesel(primary_key(match_id, download_id))]
pub struct Match {
    pub match_id: i32,
    pub download_id: i32,
    pub home_team_id: i32,
    pub home_team_name: String,
    pub away_team_id: i32,
    pub away_team_name: String,
    pub match_date: String,
    pub match_type: i32,
    pub status: String,
    pub home_goals: Option<i32>,
    pub away_goals: Option<i32>,
    /// The CHPP `MatchContextId` value. Its meaning depends on `match_type`:
    /// - League match (type 1): the `LeagueLevelUnitId` of the division
    /// - Cup/Masters/World Cup (type 3/7/…): the `CupId`
    /// - Tournament (type 50/51): the `TournamentId`
    /// - Ladder (type 62): the `LadderId`
    /// - Friendlies and other non-competition matches: 0 or NULL
    pub match_context_id: Option<i32>,
}

#[derive(Insertable)]
#[diesel(table_name = matches)]
pub struct NewMatch<'a> {
    pub match_id: i32,
    pub download_id: i32,
    pub home_team_id: i32,
    pub home_team_name: &'a str,
    pub away_team_id: i32,
    pub away_team_name: &'a str,
    pub match_date: &'a str,
    pub match_type: i32,
    pub status: &'a str,
    pub home_goals: Option<i32>,
    pub away_goals: Option<i32>,
    /// See `Match::match_context_id`.
    pub match_context_id: Option<i32>,
}

pub fn save_league_details(
    conn: &mut SqliteConnection,
    download_id: i32,
    data: &LeagueDetailsData,
) -> Result<(), Error> {
    use crate::db::schema::league_units;

    let unit_id = data.LeagueLevelUnitID as i32;

    let new_unit = NewLeagueUnit {
        unit_id,
        download_id,
        unit_name: &data.LeagueLevelUnitName,
        league_level: data.LeagueLevel as i32,
        max_number_of_teams: data.MaxLevel.map(|v| v as i32),
        current_match_round: data.CurrentMatchRound.map(|v| v as i32),
    };

    // A league unit may be fetched for multiple opponents in the same sync;
    // INSERT OR IGNORE skips the duplicate without losing any data.
    diesel::insert_or_ignore_into(league_units::table)
        .values(&new_unit)
        .execute(conn)
        .map_err(|e| Error::Io(format!("Failed to insert league unit: {}", e)))?;

    save_league_teams(conn, download_id, unit_id, data)?;

    Ok(())
}

fn save_league_teams(
    conn: &mut SqliteConnection,
    download_id: i32,
    unit_id: i32,
    data: &LeagueDetailsData,
) -> Result<(), Error> {
    use crate::db::schema::league_unit_teams;

    let records: Vec<NewLeagueUnitTeam> = data
        .Teams
        .iter()
        .map(|team| NewLeagueUnitTeam {
            unit_id,
            team_id: team.TeamID.parse::<i32>().unwrap_or(0),
            download_id,
            team_name: &team.TeamName,
            position: team.Position as i32,
            points: team.Points as i32,
            matches_played: team.Matches as i32,
            goals_for: team.GoalsFor as i32,
            goals_against: team.GoalsAgainst as i32,
            won: team.Won as i32,
            draws: team.Draws as i32,
            lost: team.Lost as i32,
        })
        .collect();

    // Deduplicate by composite PK before inserting so the plain INSERT never
    // sees a conflict — consistent with the insert-only schema invariant.
    let mut seen_pks = std::collections::HashSet::new();
    let records: Vec<NewLeagueUnitTeam> = records
        .into_iter()
        .filter(|r| seen_pks.insert((r.unit_id, r.team_id, r.download_id)))
        .collect();

    // The same league unit may be fetched multiple times in one sync;
    // INSERT OR IGNORE skips rows already present under this download_id.
    diesel::insert_or_ignore_into(league_unit_teams::table)
        .values(&records)
        .execute(conn)
        .map_err(|e| Error::Io(format!("Failed to insert league teams: {}", e)))?;

    Ok(())
}

pub fn save_matches(
    conn: &mut SqliteConnection,
    download_id: i32,
    data: &MatchesData,
) -> Result<(), Error> {
    use crate::db::schema::matches;

    let records: Vec<NewMatch> = data
        .Team
        .MatchList
        .Matches
        .iter()
        .map(|match_data| NewMatch {
            match_id: match_data.MatchID as i32,
            download_id,
            home_team_id: match_data.HomeTeam.HomeTeamID.parse::<i32>().unwrap_or(0),
            home_team_name: &match_data.HomeTeam.HomeTeamName,
            away_team_id: match_data.AwayTeam.AwayTeamID.parse::<i32>().unwrap_or(0),
            away_team_name: &match_data.AwayTeam.AwayTeamName,
            match_date: &match_data.MatchDate,
            match_type: match_data.MatchType as i32,
            status: &match_data.Status,
            home_goals: match_data.HomeGoals.map(|v| v as i32),
            away_goals: match_data.AwayGoals.map(|v| v as i32),
            match_context_id: match_data.MatchContextId.map(|v| v as i32),
        })
        .collect();

    // Deduplicate by composite PK (match_id, download_id) in Rust before
    // inserting.  The same match can appear in multiple opponents' archives
    // when they are fetched within a single download; deduplicating here
    // keeps the plain INSERT free of conflicts as required by the insert-only
    // schema invariant.
    let mut seen_ids = std::collections::HashSet::new();
    let records: Vec<NewMatch> = records
        .into_iter()
        .filter(|r| seen_ids.insert((r.match_id, r.download_id)))
        .collect();

    // The same match can appear in multiple API responses within one sync;
    // INSERT OR IGNORE skips rows already present under this download_id.
    diesel::insert_or_ignore_into(matches::table)
        .values(&records)
        .execute(conn)
        .map_err(|e| Error::Io(format!("Failed to insert matches: {}", e)))?;

    Ok(())
}

pub fn get_latest_league_details(
    conn: &mut SqliteConnection,
    league_unit_id: u32,
) -> Result<Option<LeagueDetailsData>, Error> {
    use crate::db::schema::{league_unit_teams, league_units};

    // 1. Find the latest league_unit entry for this unit_id
    let unit_opt: Option<LeagueUnit> = league_units::table
        .filter(league_units::unit_id.eq(league_unit_id as i32))
        .order(league_units::download_id.desc())
        .first(conn)
        .optional()
        .map_err(|e| Error::Io(format!("Failed to get league unit: {}", e)))?;

    if let Some(unit) = unit_opt {
        // 2. Fetch the teams using the composite FK (unit_id, download_id)
        let db_teams: Vec<LeagueUnitTeam> = league_unit_teams::table
            .filter(league_unit_teams::unit_id.eq(unit.unit_id))
            .filter(league_unit_teams::download_id.eq(unit.download_id))
            .order(league_unit_teams::position.asc())
            .load(conn)
            .map_err(|e| Error::Io(format!("Failed to get league teams: {}", e)))?;

        // 3. Convert back to CHPP model (LeagueDetailsData)
        let teams: Vec<crate::chpp::model::LeagueTeam> = db_teams
            .into_iter()
            .map(|unit_team| crate::chpp::model::LeagueTeam {
                UserId: None, // Not persisted
                TeamID: unit_team.team_id.to_string(),
                TeamName: unit_team.team_name,
                Position: unit_team.position as u32,
                PositionChange: 0, // Not persisted
                Matches: unit_team.matches_played as u32,
                GoalsFor: unit_team.goals_for as u32,
                GoalsAgainst: unit_team.goals_against as u32,
                Points: unit_team.points as u32,
                Won: unit_team.won as u32,
                Draws: unit_team.draws as u32,
                Lost: unit_team.lost as u32,
            })
            .collect();

        // Populate LeagueDetailsData with flat structure
        Ok(Some(LeagueDetailsData {
            LeagueID: 0,               // Not persisted
            LeagueName: String::new(), // Not persisted
            LeagueLevel: unit.league_level as u32,
            MaxLevel: None, // Not persisted
            LeagueLevelUnitID: unit.unit_id as u32,
            LeagueLevelUnitName: unit.unit_name,
            CurrentMatchRound: unit.current_match_round.map(|v| v as u32),
            Rank: None, // Not persisted
            Teams: teams,
        }))
    } else {
        Ok(None)
    }
}

pub fn get_latest_matches(
    conn: &mut SqliteConnection,
    team_id: u32,
) -> Result<Option<MatchesData>, Error> {
    use crate::db::schema::matches;

    // Fetch all matches involving the team
    let db_matches: Vec<Match> = matches::table
        .filter(
            matches::home_team_id
                .eq(team_id as i32)
                .or(matches::away_team_id.eq(team_id as i32)),
        )
        .order(matches::download_id.desc())
        .load(conn)
        .map_err(|e| Error::Io(format!("Failed to load matches: {}", e)))?;

    if db_matches.is_empty() {
        return Ok(None);
    }

    // Deduplicate by match_id, keeping the latest download version
    let mut unique_matches = std::collections::HashMap::new();
    for m in db_matches {
        unique_matches.entry(m.match_id).or_insert(m);
    }

    let mut matches_list: Vec<crate::chpp::model::MatchDetails> = unique_matches
        .into_values()
        .map(|match_entity| crate::chpp::model::MatchDetails {
            MatchID: match_entity.match_id as u32,
            HomeTeam: crate::chpp::model::MatchHomeTeam {
                HomeTeamID: match_entity.home_team_id.to_string(),
                HomeTeamName: match_entity.home_team_name,
                ..Default::default()
            },
            AwayTeam: crate::chpp::model::MatchAwayTeam {
                AwayTeamID: match_entity.away_team_id.to_string(),
                AwayTeamName: match_entity.away_team_name,
                ..Default::default()
            },
            MatchDate: match_entity.match_date,
            SourceSystem: None,
            MatchType: match_entity.match_type as u32,
            MatchContextId: match_entity.match_context_id.map(|v| v as u32),
            CupLevel: None,
            CupLevelIndex: None,
            HomeGoals: match_entity.home_goals.map(|v| v as u32),
            AwayGoals: match_entity.away_goals.map(|v| v as u32),
            OrdersGiven: None,
            Status: match_entity.status,
        })
        .collect();

    // Sort by date descending
    matches_list.sort_by(|a, b| b.MatchDate.cmp(&a.MatchDate));

    // Reconstruct MatchesData with Team wrapper
    Ok(Some(MatchesData {
        Team: crate::chpp::model::MatchesTeamWrapper {
            TeamID: team_id.to_string(),
            TeamName: "Unknown".to_string(),
            ShortTeamName: None,
            League: None,
            LeagueLevelUnit: None,
            MatchList: crate::chpp::model::MatchesListWrapper {
                Matches: matches_list,
            },
        },
    }))
}

/// Load all matches involving any of the given team IDs from the local DB,
/// returning one `MatchDetails` per unique `match_id` (latest download wins).
///
/// Used by the series controller to gather enough data for form computation
/// across *all* teams in a league unit rather than just the user's own team.
pub fn get_matches_for_teams(
    conn: &mut SqliteConnection,
    team_ids: &[i32],
) -> Result<Vec<crate::chpp::model::MatchDetails>, Error> {
    use crate::db::schema::matches;

    if team_ids.is_empty() {
        return Ok(Vec::new());
    }

    let db_matches: Vec<Match> = matches::table
        .filter(
            matches::home_team_id
                .eq_any(team_ids)
                .or(matches::away_team_id.eq_any(team_ids)),
        )
        .order(matches::download_id.desc())
        .load(conn)
        .map_err(|e| Error::Io(format!("Failed to load team matches: {}", e)))?;

    // Keep only the most recent download per match_id
    let mut unique_matches: std::collections::HashMap<i32, Match> =
        std::collections::HashMap::new();
    for m in db_matches {
        unique_matches.entry(m.match_id).or_insert(m);
    }

    let details: Vec<crate::chpp::model::MatchDetails> = unique_matches
        .into_values()
        .map(|m| crate::chpp::model::MatchDetails {
            MatchID: m.match_id as u32,
            HomeTeam: crate::chpp::model::MatchHomeTeam {
                HomeTeamID: m.home_team_id.to_string(),
                HomeTeamName: m.home_team_name,
                ..Default::default()
            },
            AwayTeam: crate::chpp::model::MatchAwayTeam {
                AwayTeamID: m.away_team_id.to_string(),
                AwayTeamName: m.away_team_name,
                ..Default::default()
            },
            MatchDate: m.match_date,
            SourceSystem: None,
            MatchType: m.match_type as u32,
            MatchContextId: m.match_context_id.map(|v| v as u32),
            CupLevel: None,
            CupLevelIndex: None,
            HomeGoals: m.home_goals.map(|v| v as u32),
            AwayGoals: m.away_goals.map(|v| v as u32),
            OrdersGiven: None,
            Status: m.status,
        })
        .collect();

    Ok(details)
}

pub fn get_upcoming_opponents_from_db(
    conn: &mut SqliteConnection,
    our_team_id: u32,
) -> Result<Vec<crate::service::opponent_analysis::UpcomingOpponent>, Error> {
    use crate::db::schema::matches;

    // Find all upcoming matches involving our team
    let db_matches: Vec<Match> = matches::table
        .filter(matches::status.eq("UPCOMING"))
        .filter(
            matches::home_team_id
                .eq(our_team_id as i32)
                .or(matches::away_team_id.eq(our_team_id as i32)),
        )
        .order(matches::download_id.desc())
        .load(conn)
        .map_err(|e| Error::Io(format!("Failed to load upcoming matches: {}", e)))?;

    // Deduplicate by match_id
    let mut unique_matches = std::collections::HashMap::new();
    for m in db_matches {
        unique_matches.entry(m.match_id).or_insert(m);
    }

    let mut opponents = Vec::new();
    for m in unique_matches.into_values() {
        let home_id = m.home_team_id as u32;
        let (opp_id, opp_name) = if home_id == our_team_id {
            (m.away_team_id as u32, m.away_team_name)
        } else {
            (home_id, m.home_team_name)
        };

        opponents.push(crate::service::opponent_analysis::UpcomingOpponent {
            team_id: opp_id,
            team_name: opp_name,
            match_date: m.match_date,
            match_id: m.match_id as u32,
        });
    }

    // Sort ascending by date for upcoming matches
    opponents.sort_by(|a, b| a.match_date.cmp(&b.match_date));

    Ok(opponents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::download_entries::NewDownload;
    use crate::db::schema::downloads;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    use serial_test::serial;

    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    fn establish_connection() -> SqliteConnection {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("Error connecting to :memory: database");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Error running migrations");
        conn
    }

    #[test]
    #[serial]
    fn test_series_persistence() {
        let mut conn = establish_connection();

        // Create a download record
        diesel::insert_into(downloads::table)
            .values(NewDownload {
                timestamp: "2026-02-15T12:00:00Z".to_string(),
                status: "completed".to_string(),
            })
            .execute(&mut conn)
            .expect("Failed to create download");

        // Mock League Data
        let league_data = LeagueDetailsData {
            LeagueID: 0,
            LeagueName: "Test League".to_string(),
            LeagueLevel: 5,
            MaxLevel: Some(8),
            LeagueLevelUnitID: 100,
            LeagueLevelUnitName: "Test League Unit".to_string(),
            CurrentMatchRound: Some(1),
            Rank: None,
            Teams: vec![crate::chpp::model::LeagueTeam {
                UserId: None,
                TeamID: "1".to_string(),
                TeamName: "Team A".to_string(),
                Position: 1,
                PositionChange: 0,
                Matches: 1,
                GoalsFor: 2,
                GoalsAgainst: 0,
                Points: 3,
                Won: 1,
                Draws: 0,
                Lost: 0,
            }],
        };

        save_league_details(&mut conn, 1, &league_data).expect("Failed to save league details");

        // Verify League Data
        let fetched_league = get_latest_league_details(&mut conn, 100)
            .expect("Failed to fetch league")
            .unwrap();
        assert_eq!(fetched_league.LeagueLevelUnitID, 100);
        assert_eq!(fetched_league.Teams.len(), 1);
        assert_eq!(fetched_league.Teams[0].TeamName, "Team A");

        // Idempotency: re-saving must not duplicate rows (ON CONFLICT DO NOTHING)
        save_league_details(&mut conn, 1, &league_data).expect("Failed to re-save league details");
        let fetched_again = get_latest_league_details(&mut conn, 100)
            .expect("Failed to fetch league after re-save")
            .unwrap();
        assert_eq!(fetched_again.Teams.len(), 1);

        // Mock Match Data
        // Mock Match Data - Download 1
        let match_data = MatchesData {
            Team: crate::chpp::model::MatchesTeamWrapper {
                TeamID: "1".to_string(),
                TeamName: "Team A".to_string(),
                ShortTeamName: None,
                League: None,
                LeagueLevelUnit: None,
                MatchList: crate::chpp::model::MatchesListWrapper {
                    Matches: vec![crate::chpp::model::MatchDetails {
                        MatchID: 500,
                        HomeTeam: crate::chpp::model::MatchHomeTeam {
                            HomeTeamID: "1".to_string(),
                            HomeTeamName: "Team A".to_string(),
                            ..Default::default()
                        },
                        AwayTeam: crate::chpp::model::MatchAwayTeam {
                            AwayTeamID: "2".to_string(),
                            AwayTeamName: "Team B".to_string(),
                            ..Default::default()
                        },
                        MatchDate: "2026-02-15 14:00:00".to_string(),
                        SourceSystem: None,
                        MatchType: 1,
                        Status: "FINISHED".to_string(),
                        MatchContextId: None,
                        CupLevel: None,
                        CupLevelIndex: None,
                        HomeGoals: Some(2),
                        AwayGoals: Some(1),
                        OrdersGiven: None,
                    }],
                },
            },
        };

        save_matches(&mut conn, 1, &match_data).expect("Failed to save matches");

        // Idempotency: re-saving matches must not duplicate rows
        save_matches(&mut conn, 1, &match_data).expect("Failed to re-save matches");

        // Verify Match Data
        // Mock Match Data - Download 2 (Updated goals for same match_id 500)
        let match_data_v2 = MatchesData {
            Team: crate::chpp::model::MatchesTeamWrapper {
                TeamID: "1".to_string(),
                TeamName: "Team A Updated".to_string(),
                ShortTeamName: None,
                League: None,
                LeagueLevelUnit: None,
                MatchList: crate::chpp::model::MatchesListWrapper {
                    Matches: vec![
                        crate::chpp::model::MatchDetails {
                            MatchID: 500,
                            HomeTeam: crate::chpp::model::MatchHomeTeam {
                                HomeTeamID: "1".to_string(),
                                HomeTeamName: "Team A Updated".to_string(),
                                ..Default::default()
                            },
                            AwayTeam: crate::chpp::model::MatchAwayTeam {
                                AwayTeamID: "2".to_string(),
                                AwayTeamName: "Team B".to_string(),
                                ..Default::default()
                            },
                            MatchDate: "2026-02-15 14:00:00".to_string(),
                            SourceSystem: None,
                            MatchType: 1,
                            Status: "FINISHED".to_string(),
                            MatchContextId: None,
                            CupLevel: None,
                            CupLevelIndex: None,
                            HomeGoals: Some(3), // Updated
                            AwayGoals: Some(1),
                            OrdersGiven: None,
                        },
                        crate::chpp::model::MatchDetails {
                            MatchID: 501, // New match
                            HomeTeam: crate::chpp::model::MatchHomeTeam {
                                HomeTeamID: "1".to_string(),
                                HomeTeamName: "Team A Updated".to_string(),
                                ..Default::default()
                            },
                            AwayTeam: crate::chpp::model::MatchAwayTeam {
                                AwayTeamID: "3".to_string(),
                                AwayTeamName: "Team C".to_string(),
                                ..Default::default()
                            },
                            MatchDate: "2026-02-22 14:00:00".to_string(),
                            SourceSystem: None,
                            MatchType: 1,
                            Status: "FINISHED".to_string(),
                            MatchContextId: None,
                            CupLevel: None,
                            CupLevelIndex: None,
                            HomeGoals: Some(1),
                            AwayGoals: Some(1),
                            OrdersGiven: None,
                        },
                    ],
                },
            },
        };

        save_matches(&mut conn, 2, &match_data_v2).expect("Failed to save matches v2");

        // Verify Match Data - Should get 2 matches, with match 500 being the one from download 2
        let fetched_matches = get_latest_matches(&mut conn, 1)
            .expect("Failed to fetch matches")
            .unwrap();
        assert_eq!(fetched_matches.Team.MatchList.Matches.len(), 2);

        let m500 = fetched_matches
            .Team
            .MatchList
            .Matches
            .iter()
            .find(|m| m.MatchID == 500)
            .unwrap();
        assert_eq!(m500.HomeGoals, Some(3));
        assert_eq!(m500.HomeTeam.HomeTeamName, "Team A Updated");

        let m501 = fetched_matches
            .Team
            .MatchList
            .Matches
            .iter()
            .find(|m| m.MatchID == 501)
            .unwrap();
        assert_eq!(m501.MatchID, 501);

        // Verify upcoming opponents from DB
        // Add an UPCOMING match
        let upcoming_data = MatchesData {
            Team: crate::chpp::model::MatchesTeamWrapper {
                TeamID: "1".to_string(),
                TeamName: "Team A".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        // Explicitly constructing as MatchesListWrapper is easier for the test helper
        let mut m_list = Vec::new();
        m_list.push(crate::chpp::model::MatchDetails {
            MatchID: 600,
            HomeTeam: crate::chpp::model::MatchHomeTeam {
                HomeTeamID: "1".to_string(),
                HomeTeamName: "Team A".to_string(),
                ..Default::default()
            },
            AwayTeam: crate::chpp::model::MatchAwayTeam {
                AwayTeamID: "4".to_string(),
                AwayTeamName: "Team D".to_string(),
                ..Default::default()
            },
            MatchDate: "2026-03-01 14:00:00".to_string(),
            Status: "UPCOMING".to_string(),
            ..crate::chpp::model::MatchDetails::default()
        });

        let full_upcoming_data = MatchesData {
            Team: crate::chpp::model::MatchesTeamWrapper {
                TeamID: "1".to_string(),
                TeamName: "Team A".to_string(),
                MatchList: crate::chpp::model::MatchesListWrapper { Matches: m_list },
                ..Default::default()
            },
        };

        save_matches(&mut conn, 3, &full_upcoming_data).expect("Failed to save upcoming match");

        let upcoming =
            get_upcoming_opponents_from_db(&mut conn, 1).expect("Failed to get upcoming");
        assert_eq!(upcoming.len(), 1);
        assert_eq!(upcoming[0].team_id, 4);
        assert_eq!(upcoming[0].team_name, "Team D");
    }
}
