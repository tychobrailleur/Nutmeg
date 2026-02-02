/* teams.rs
 *
 * Copyright 2026 sebastien
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

use crate::chpp::error::Error;
use crate::chpp::model::{
    Country, Cup, Currency, Language, League, Region, SupporterTier, Team, User, WorldDetails,
};
use crate::db::schema::{
    countries, cups, currencies, downloads, languages, leagues, players, regions, teams, users,
};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

#[derive(Queryable, Insertable)]
#[diesel(table_name = languages)]
struct LanguageEntity {
    id: i32,
    name: String,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = currencies)]
struct CurrencyEntity {
    id: i32,
    name: String,
    rate: Option<f64>,
    symbol: Option<String>,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = users)]
struct UserEntity {
    id: i32,
    name: String,
    login_name: String,
    supporter_tier: String,
    signup_date: Option<String>,
    activation_date: Option<String>,
    last_login_date: Option<String>,
    has_manager_license: Option<bool>,
    language_id: Option<i32>,
    language_name: Option<String>,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = countries)]
struct CountryEntity {
    id: i32,
    name: String,
    currency_id: Option<i32>,
    country_code: Option<String>,
    date_format: Option<String>,
    time_format: Option<String>,
    flag: Option<String>,
}

fn get_flag_emoji(country_code: Option<&str>) -> Option<String> {
    let code = country_code?;
    if code.len() != 2 {
        return None;
    }
    let mut s = String::new();

    // https://en.wikipedia.org/wiki/Regional_indicator_symbol
    // A country flag is a pair of regional indicator symbols,
    // which together become the emoji flag sequence.
    // Regional indicator A is U+1F1E6 (https://www.compart.com/en/unicode/U+1F1E6),
    // which corresponds to codepoint 127462 (uppercase A is 65)
    // So to get regional indicator corresponding to the uppercase letter,
    // we shift the codepoint by 127462-65 = 127397.
    for c in code.to_uppercase().chars() {
        if c < 'A' || c > 'Z' {
            return None;
        }
        let u = c as u32 + 127_397;
        if let Some(ch) = std::char::from_u32(u) {
            s.push(ch);
        } else {
            return None;
        }
    }
    Some(s)
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = regions)]
struct RegionEntity {
    id: i32,
    name: String,
    country_id: i32,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = leagues)]
struct LeagueEntity {
    id: i32,
    name: String,
    country_id: Option<i32>,
    short_name: Option<String>,
    continent: Option<String>,
    season: Option<i32>,
    season_offset: Option<i32>,
    match_round: Option<i32>,
    zone_name: Option<String>,
    english_name: Option<String>,
    language_id: Option<i32>,
    national_team_id: Option<i32>,
    u20_team_id: Option<i32>,
    active_teams: Option<i32>,
    active_users: Option<i32>,
    number_of_levels: Option<i32>,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = cups)]
struct CupEntity {
    id: i32,
    name: String,
    league_level: Option<i32>,
    level: Option<i32>,
    level_index: Option<i32>,
    match_round: Option<i32>,
    match_rounds_left: Option<i32>,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = downloads)]
pub struct DownloadEntity {
    pub id: i32,
    pub timestamp: String,
    pub status: String,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = teams)]
struct TeamEntity {
    download_id: i32,
    id: i32,
    user_id: Option<i32>,
    name: String,
    raw_data: String,
    short_name: Option<String>,
    is_primary_club: Option<bool>,
    founded_date: Option<String>,
    arena_id: Option<i32>,
    arena_name: Option<String>,
    league_id: Option<i32>,
    league_name: Option<String>,
    country_id: Option<i32>,
    country_name: Option<String>,
    region_id: Option<i32>,
    region_name: Option<String>,
    homepage: Option<String>,
    dress_uri: Option<String>,
    dress_alternate_uri: Option<String>,
    logo_url: Option<String>,
    trainer_id: Option<i32>,
    cup_still_in: Option<bool>,
    cup_id: Option<i32>,
    cup_name: Option<String>,
    cup_league_level: Option<i32>,
    cup_level: Option<i32>,
    cup_level_index: Option<i32>,
    cup_match_round: Option<i32>,
    cup_match_rounds_left: Option<i32>,
    power_rating_global: Option<i32>,
    power_rating_league: Option<i32>,
    power_rating_region: Option<i32>,
    power_rating_indiv: Option<i32>,
    friendly_team_id: Option<i32>,
    league_level_unit_id: Option<i32>,
    league_level_unit_name: Option<String>,
    league_level: Option<i32>,
    number_of_victories: Option<i32>,
    number_of_undefeated: Option<i32>,
    number_of_visits: Option<i32>,
    team_rank: Option<i32>,
    fanclub_id: Option<i32>,
    fanclub_name: Option<String>,
    fanclub_size: Option<i32>,
    color_background: Option<String>,
    color_primary: Option<String>,
    is_bot: Option<bool>,
    bot_since: Option<String>,
    youth_team_id: Option<i32>,
    youth_team_name: Option<String>,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = players)]
struct PlayerEntity {
    id: i32,
    download_id: i32,
    team_id: i32,
    first_name: String,
    last_name: String,
    player_number: i32,
    age: i32,
    age_days: Option<i32>,
    tsi: i32,
    player_form: i32,
    statement: Option<String>,
    experience: i32,
    loyalty: i32,
    mother_club_bonus: bool,
    leadership: i32,
    salary: i32,
    is_abroad: bool,
    agreeability: i32,
    aggressiveness: i32,
    honesty: i32,
    league_goals: Option<i32>,
    cup_goals: Option<i32>,
    friendlies_goals: Option<i32>,
    career_goals: Option<i32>,
    career_hattricks: Option<i32>,
    speciality: Option<i32>,
    transfer_listed: bool,
    national_team_id: Option<i32>,
    country_id: i32,
    caps: Option<i32>,
    caps_u20: Option<i32>,
    cards: Option<i32>,
    injury_level: Option<i32>,
    sticker: Option<String>,
    stamina_skill: Option<i32>,
    keeper_skill: Option<i32>,
    playmaker_skill: Option<i32>,
    scorer_skill: Option<i32>,
    passing_skill: Option<i32>,
    winger_skill: Option<i32>,
    defender_skill: Option<i32>,
    set_pieces_skill: Option<i32>,
    last_match_date: Option<String>,
    last_match_id: Option<i32>,
    last_match_position_code: Option<i32>,
    last_match_played_minutes: Option<i32>,
    last_match_rating: Option<i32>,
    last_match_rating_end_of_match: Option<i32>,
}

pub fn save_world_details(
    conn: &mut SqliteConnection,
    world_details: &WorldDetails,
) -> Result<(), Error> {
    for world_league in &world_details.LeagueList.Leagues {
        // Save Language if present
        if let (Some(lang_id), Some(lang_name)) =
            (world_league.LanguageID, &world_league.LanguageName)
        {
            let language = Language {
                LanguageID: lang_id,
                LanguageName: lang_name.clone(),
            };
            save_language(conn, &language)?;
        }

        // Save Currency if present in WorldCountry
        // WorldCountry has CurrencyName and CurrencyRate but no CurrencyID
        // We'll use CountryID as a proxy for CurrencyID since each country has one currency
        if let (Some(country_id), Some(currency_name), Some(currency_rate_str)) = (
            world_league.Country.CountryID,
            &world_league.Country.CurrencyName,
            &world_league.Country.CurrencyRate,
        ) {
            // Parse rate (handle comma as decimal separator)
            let rate = currency_rate_str.replace(',', ".").parse::<f64>().ok();

            let currency = Currency {
                CurrencyID: country_id, // Using country ID as currency ID
                CurrencyName: currency_name.clone(),
                Rate: rate,
                Symbol: Some(currency_name.clone()), // Using name as symbol for now
            };
            save_currency(conn, &currency)?;
        }

        // Construct a standard League object from WorldLeague data
        let league = League {
            LeagueID: world_league.LeagueID,
            LeagueName: world_league.LeagueName.clone(),
            ShortName: world_league.ShortName.clone(),
            Continent: world_league.Continent.clone(),
            Season: world_league.Season,
            SeasonOffset: world_league.SeasonOffset,
            MatchRound: world_league.MatchRound,
            ZoneName: world_league.ZoneName.clone(),
            EnglishName: world_league.EnglishName.clone(),
            LanguageID: world_league.LanguageID,
            NationalTeamId: world_league.NationalTeamId,
            U20TeamId: world_league.U20TeamId,
            ActiveTeams: world_league.ActiveTeams,
            ActiveUsers: world_league.ActiveUsers,
            NumberOfLevels: world_league.NumberOfLevels,
        };

        // Save Country
        if let Some(country_id) = world_league.Country.CountryID {
            let country_model = Country {
                CountryID: country_id,
                CountryName: world_league.Country.CountryName.clone().unwrap_or_default(),
                Currency: None, // Currency is saved separately above
                CountryCode: world_league.Country.CountryCode.clone(),
                DateFormat: world_league.Country.DateFormat.clone(),
                TimeFormat: world_league.Country.TimeFormat.clone(),
            };
            save_country(conn, &country_model)?;
        }

        // Save the League
        save_league(conn, &league, world_league.Country.CountryID)?;
    }
    Ok(())
}

pub fn save_players(
    conn: &mut SqliteConnection,
    players_list: &[crate::chpp::model::Player],
    team_id: u32,
    download_id: i32,
) -> Result<(), Error> {
    for player in players_list {
        let entity = PlayerEntity {
            id: player.PlayerID as i32,
            download_id,
            team_id: team_id as i32,
            first_name: player.FirstName.clone(),
            last_name: player.LastName.clone(),
            player_number: player.PlayerNumber.unwrap_or(100) as i32,
            age: player.Age as i32,
            age_days: player.AgeDays.map(|v| v as i32),
            tsi: player.TSI as i32,
            player_form: player.PlayerForm as i32,
            statement: player.Statement.clone(),
            experience: player.Experience as i32,
            loyalty: player.Loyalty as i32,
            mother_club_bonus: player.MotherClubBonus,
            leadership: player.Leadership as i32,
            salary: player.Salary as i32,
            is_abroad: player.IsAbroad,
            agreeability: player.Agreeability as i32,
            aggressiveness: player.Aggressiveness as i32,
            honesty: player.Honesty as i32,
            league_goals: player.LeagueGoals.map(|v| v as i32),
            cup_goals: player.CupGoals.map(|v| v as i32),
            friendlies_goals: player.FriendliesGoals.map(|v| v as i32),
            career_goals: player.CareerGoals.map(|v| v as i32),
            career_hattricks: player.CareerHattricks.map(|v| v as i32),
            speciality: player.Speciality.map(|v| v as i32),
            transfer_listed: player.TransferListed,
            national_team_id: player.NationalTeamID.map(|v| v as i32),
            country_id: player.CountryID.unwrap_or(0) as i32,
            caps: player.Caps.map(|v| v as i32),
            caps_u20: player.CapsU20.map(|v| v as i32),
            cards: player.Cards.map(|v| v as i32),
            injury_level: player.InjuryLevel.map(|v| v as i32),
            sticker: player.Sticker.clone(),
            // Skills
            stamina_skill: player.PlayerSkills.as_ref().map(|s| s.StaminaSkill as i32),
            keeper_skill: player.PlayerSkills.as_ref().map(|s| s.KeeperSkill as i32),
            playmaker_skill: player
                .PlayerSkills
                .as_ref()
                .map(|s| s.PlaymakerSkill as i32),
            scorer_skill: player.PlayerSkills.as_ref().map(|s| s.ScorerSkill as i32),
            passing_skill: player.PlayerSkills.as_ref().map(|s| s.PassingSkill as i32),
            winger_skill: player.PlayerSkills.as_ref().map(|s| s.WingerSkill as i32),
            defender_skill: player.PlayerSkills.as_ref().map(|s| s.DefenderSkill as i32),
            set_pieces_skill: player
                .PlayerSkills
                .as_ref()
                .map(|s| s.SetPiecesSkill as i32),
            // Last Match
            last_match_date: player.LastMatch.as_ref().map(|m| m.Date.clone()),
            last_match_id: player.LastMatch.as_ref().map(|m| m.MatchId as i32),
            last_match_position_code: player.LastMatch.as_ref().map(|m| m.PositionCode as i32),
            last_match_played_minutes: player.LastMatch.as_ref().map(|m| m.PlayedMinutes as i32),
            last_match_rating: player
                .LastMatch
                .as_ref()
                .and_then(|m| m.Rating.map(|v| v as i32)),
            last_match_rating_end_of_match: player
                .LastMatch
                .as_ref()
                .and_then(|m| m.RatingEndOfMatch.map(|v| v as i32)),
        };

        diesel::insert_into(players::table)
            .values(&entity)
            .on_conflict((players::id, players::download_id))
            .do_nothing()
            .execute(conn)
            .map_err(|e| Error::Io(format!("Database error saving player: {}", e)))?;
    }
    Ok(())
}

// Persists a Language entity.
// We use ON CONFLICT DO UPDATE to handle cases where the language already exists
// but might have a different name (though unlikely for IDs).
fn save_language(conn: &mut SqliteConnection, language: &Language) -> Result<(), Error> {
    let entity = LanguageEntity {
        id: language.LanguageID as i32,
        name: language.LanguageName.clone(),
    };
    diesel::insert_into(languages::table)
        .values(&entity)
        .on_conflict(languages::id)
        .do_update()
        .set(languages::name.eq(&entity.name))
        .execute(conn)
        .map_err(|e| Error::Io(format!("Database error saving language: {}", e)))?;
    Ok(())
}

// Persists a Currency entity.
// Rate and Symbol are optional and updated if the currency ID exists.
fn save_currency(conn: &mut SqliteConnection, currency: &Currency) -> Result<(), Error> {
    let entity = CurrencyEntity {
        id: currency.CurrencyID as i32,
        name: currency.CurrencyName.clone(),
        rate: currency.Rate,
        symbol: currency.Symbol.clone(),
    };
    diesel::insert_into(currencies::table)
        .values(&entity)
        .on_conflict(currencies::id)
        .do_update()
        .set((
            currencies::name.eq(&entity.name),
            currencies::rate.eq(&entity.rate),
            currencies::symbol.eq(&entity.symbol),
        ))
        .execute(conn)
        .map_err(|e| Error::Io(format!("Database error saving currency: {}", e)))?;
    Ok(())
}

// Persists a User and their associated Language.
// This function acts as an aggregate root saver for User data.
fn save_user(conn: &mut SqliteConnection, user: &User) -> Result<(), Error> {
    // Save Language first to ensure the Foreign Key in 'users' is valid.
    save_language(conn, &user.Language)?;

    let supporter_tier_str = format!("{:?}", user.SupporterTier);

    let entity = UserEntity {
        id: user.UserID as i32,
        name: user.Name.clone(),
        login_name: user.Loginname.clone(),
        supporter_tier: supporter_tier_str,
        signup_date: Some(user.SignupDate.clone()),
        activation_date: Some(user.ActivationDate.clone()),
        last_login_date: Some(user.LastLoginDate.clone()),
        has_manager_license: Some(user.HasManagerLicense),
        language_id: Some(user.Language.LanguageID as i32),
        language_name: Some(user.Language.LanguageName.clone()),
    };

    diesel::insert_into(users::table)
        .values(&entity)
        .on_conflict(users::id)
        .do_update()
        .set((
            users::name.eq(&entity.name),
            users::login_name.eq(&entity.login_name),
            users::supporter_tier.eq(&entity.supporter_tier),
            users::last_login_date.eq(&entity.last_login_date),
            users::has_manager_license.eq(&entity.has_manager_license),
            users::language_id.eq(&entity.language_id),
            users::language_name.eq(&entity.language_name),
        ))
        .execute(conn)
        .map_err(|e| Error::Io(format!("Database error saving user: {}", e)))?;
    Ok(())
}

// Persists a Country and its optional Currency.
fn save_country(conn: &mut SqliteConnection, country: &Country) -> Result<(), Error> {
    if let Some(c) = &country.Currency {
        save_currency(conn, c)?;
    }

    let flag = get_flag_emoji(country.CountryCode.as_deref());

    let entity = CountryEntity {
        id: country.CountryID as i32,
        name: country.CountryName.clone(),
        currency_id: country.Currency.as_ref().map(|c| c.CurrencyID as i32),
        country_code: country.CountryCode.clone(),
        date_format: country.DateFormat.clone(),
        time_format: country.TimeFormat.clone(),
        flag,
    };
    diesel::insert_into(countries::table)
        .values(&entity)
        .on_conflict(countries::id)
        .do_update()
        .set((
            countries::name.eq(&entity.name),
            countries::currency_id.eq(&entity.currency_id),
            countries::country_code.eq(&entity.country_code),
            countries::date_format.eq(&entity.date_format),
            countries::time_format.eq(&entity.time_format),
        ))
        .execute(conn)
        .map_err(|e| Error::Io(format!("Database error saving country: {}", e)))?;
    Ok(())
}

// Persists a Region, linking it to its parent Country.
fn save_region(
    conn: &mut SqliteConnection,
    region: &Region,
    country_id_opt: Option<u32>,
) -> Result<(), Error> {
    if let Some(c_id) = country_id_opt {
        let entity = RegionEntity {
            id: region.RegionID as i32,
            name: region.RegionName.clone(),
            country_id: c_id as i32,
        };
        diesel::insert_into(regions::table)
            .values(&entity)
            .on_conflict(regions::id)
            .do_update()
            .set((
                regions::name.eq(&entity.name),
                regions::country_id.eq(&entity.country_id),
            ))
            .execute(conn)
            .map_err(|e| Error::Io(format!("Database error saving region: {}", e)))?;
    }

    Ok(())
}

// Persists a League, optionally linking it to a Country.
fn save_league(
    conn: &mut SqliteConnection,
    league: &League,
    country_id_opt: Option<u32>,
) -> Result<(), Error> {
    let entity = LeagueEntity {
        id: league.LeagueID as i32,
        name: league.LeagueName.clone(),
        country_id: country_id_opt.map(|id| id as i32),
        short_name: league.ShortName.clone(),
        continent: league.Continent.clone(),
        season: league.Season.map(|v| v as i32),
        season_offset: league.SeasonOffset.map(|v| v as i32),
        match_round: league.MatchRound.map(|v| v as i32),
        zone_name: league.ZoneName.clone(),
        english_name: league.EnglishName.clone(),
        language_id: league.LanguageID.map(|v| v as i32),
        national_team_id: league.NationalTeamId.map(|v| v as i32),
        u20_team_id: league.U20TeamId.map(|v| v as i32),
        active_teams: league.ActiveTeams.map(|v| v as i32),
        active_users: league.ActiveUsers.map(|v| v as i32),
        number_of_levels: league.NumberOfLevels.map(|v| v as i32),
    };
    diesel::insert_into(leagues::table)
        .values(&entity)
        .on_conflict(leagues::id)
        .do_update()
        .set((
            leagues::name.eq(&entity.name),
            leagues::country_id.eq(&entity.country_id),
            leagues::short_name.eq(&entity.short_name),
            leagues::continent.eq(&entity.continent),
            leagues::season.eq(&entity.season),
            leagues::season_offset.eq(&entity.season_offset),
            leagues::match_round.eq(&entity.match_round),
            leagues::zone_name.eq(&entity.zone_name),
            leagues::english_name.eq(&entity.english_name),
            leagues::language_id.eq(&entity.language_id),
            leagues::national_team_id.eq(&entity.national_team_id),
            leagues::u20_team_id.eq(&entity.u20_team_id),
            leagues::active_teams.eq(&entity.active_teams),
            leagues::active_users.eq(&entity.active_users),
            leagues::number_of_levels.eq(&entity.number_of_levels),
        ))
        .execute(conn)
        .map_err(|e| Error::Io(format!("Database error saving league: {}", e)))?;
    Ok(())
}

// Persists Cup details.
fn save_cup(conn: &mut SqliteConnection, cup: &Cup) -> Result<(), Error> {
    if let (Some(id), Some(name)) = (cup.CupID, &cup.CupName) {
        let entity = CupEntity {
            id: id as i32,
            name: name.clone(),
            league_level: cup.CupLeagueLevel.map(|v| v as i32),
            level: cup.CupLevel.map(|v| v as i32),
            level_index: cup.CupLevelIndex.map(|v| v as i32),
            match_round: cup.MatchRound.map(|v| v as i32),
            match_rounds_left: cup.MatchRoundsLeft.map(|v| v as i32),
        };
        diesel::insert_into(cups::table)
            .values(&entity)
            .on_conflict(cups::id)
            .do_update()
            .set((
                cups::name.eq(&entity.name),
                cups::match_round.eq(&entity.match_round),
            ))
            .execute(conn)
            .map_err(|e| Error::Io(format!("Database error saving cup: {}", e)))?;
    }
    Ok(())
}

/// Orchestrates the saving of a Team and all its related reference data.
/// This acts as the main entry point for persisting team details, ensuring
/// that dependencies (User, Country, Region, League, Cup) are saved first
/// to satisfy Foreign Key constraints.
pub fn save_team(
    conn: &mut SqliteConnection,
    team: &Team,
    user: &User,
    download_id: i32,
) -> Result<(), Error> {
    save_user(conn, user)?;

    if let Some(c) = &team.Country {
        save_country(conn, c)?;
    }

    let country_id = team.Country.as_ref().map(|c| c.CountryID);

    if let Some(r) = &team.Region {
        save_region(conn, r, country_id)?;
    }

    if let Some(l) = &team.League {
        save_league(conn, l, country_id)?;
    }

    if let Some(c) = &team.Cup {
        save_cup(conn, c)?;
    }

    let team_id_num = team
        .TeamID
        .parse::<i32>()
        .map_err(|e| Error::Parse(format!("Invalid TeamID: {}", e)))?;

    let json_data = serde_json::to_string(team)
        .map_err(|e| Error::Parse(format!("Failed to serialize team: {}", e)))?;

    let entity = TeamEntity {
        download_id,
        id: team_id_num,
        user_id: Some(user.UserID as i32),
        name: team.TeamName.clone(),
        raw_data: json_data,
        short_name: team.ShortTeamName.clone(),
        is_primary_club: team.IsPrimaryClub,
        founded_date: team.FoundedDate.clone(),
        arena_id: team.Arena.as_ref().map(|a| a.ArenaID as i32),
        arena_name: team.Arena.as_ref().map(|a| a.ArenaName.clone()),
        league_id: team.League.as_ref().map(|l| l.LeagueID as i32),
        league_name: team.League.as_ref().map(|l| l.LeagueName.clone()),
        country_id: team.Country.as_ref().map(|c| c.CountryID as i32),
        country_name: team.Country.as_ref().map(|c| c.CountryName.clone()),
        region_id: team.Region.as_ref().map(|r| r.RegionID as i32),
        region_name: team.Region.as_ref().map(|r| r.RegionName.clone()),
        homepage: team.HomePage.clone(),
        dress_uri: team.DressURI.clone(),
        dress_alternate_uri: team.DressAlternateURI.clone(),
        logo_url: team.LogoURL.clone(),
        trainer_id: team.Trainer.as_ref().map(|t| t.PlayerID as i32),
        cup_still_in: team.Cup.as_ref().and_then(|c| c.StillInCup),
        cup_id: team.Cup.as_ref().and_then(|c| c.CupID.map(|v| v as i32)),
        cup_name: team.Cup.as_ref().and_then(|c| c.CupName.clone()),
        cup_league_level: team
            .Cup
            .as_ref()
            .and_then(|c| c.CupLeagueLevel.map(|v| v as i32)),
        cup_level: team.Cup.as_ref().and_then(|c| c.CupLevel.map(|v| v as i32)),
        cup_level_index: team
            .Cup
            .as_ref()
            .and_then(|c| c.CupLevelIndex.map(|v| v as i32)),
        cup_match_round: team
            .Cup
            .as_ref()
            .and_then(|c| c.MatchRound.map(|v| v as i32)),
        cup_match_rounds_left: team
            .Cup
            .as_ref()
            .and_then(|c| c.MatchRoundsLeft.map(|v| v as i32)),
        power_rating_global: team.PowerRating.as_ref().map(|p| p.GlobalRanking as i32),
        power_rating_league: team.PowerRating.as_ref().map(|p| p.LeagueRanking as i32),
        power_rating_region: team.PowerRating.as_ref().map(|p| p.RegionRanking as i32),
        power_rating_indiv: team.PowerRating.as_ref().map(|p| p.PowerRating as i32),
        friendly_team_id: team.FriendlyTeamID.map(|v| v as i32),
        league_level_unit_id: team
            .LeagueLevelUnit
            .as_ref()
            .map(|l| l.LeagueLevelUnitID as i32),
        league_level_unit_name: team
            .LeagueLevelUnit
            .as_ref()
            .map(|l| l.LeagueLevelUnitName.clone()),
        league_level: team.LeagueLevelUnit.as_ref().map(|l| l.LeagueLevel as i32),
        number_of_victories: team.NumberOfVictories.map(|v| v as i32),
        number_of_undefeated: team.NumberOfUndefeated.map(|v| v as i32),
        number_of_visits: team.NumberOfVisits.map(|v| v as i32),
        team_rank: team.TeamRank.map(|v| v as i32),
        fanclub_id: team.Fanclub.as_ref().map(|f| f.FanclubID as i32),
        fanclub_name: team.Fanclub.as_ref().map(|f| f.FanclubName.clone()),
        fanclub_size: team.Fanclub.as_ref().map(|f| f.FanclubSize as i32),
        color_background: team.TeamColors.as_ref().map(|c| c.BackgroundColor.clone()),
        color_primary: team.TeamColors.as_ref().map(|c| c.Color.clone()),
        is_bot: team.BotStatus.as_ref().map(|b| b.IsBot),
        bot_since: team.BotStatus.as_ref().and_then(|b| b.BotSince.clone()),
        youth_team_id: team.YouthTeamID.map(|v| v as i32),
        youth_team_name: team.YouthTeamName.clone(),
    };

    diesel::insert_into(teams::table)
        .values(&entity)
        .on_conflict((teams::id, teams::download_id))
        .do_nothing()
        .execute(conn)
        .map_err(|e| Error::Io(format!("Database error: {}", e)))?;

    Ok(())
}

// Returns the ID of the most recent completed download, or None if no downloads exist
pub fn get_latest_download_id(conn: &mut SqliteConnection) -> Result<Option<i32>, Error> {
    use crate::db::schema::downloads::dsl::*;
    use diesel::prelude::*;

    downloads
        .filter(status.eq("completed"))
        .select(id)
        .order(id.desc())
        .first::<i32>(conn)
        .optional()
        .map_err(|e| Error::Db(format!("Failed to get latest download: {}", e)))
}

// Returns a list of (TeamID, TeamName) for all teams in the DB.
pub fn get_teams_summary(conn: &mut SqliteConnection) -> Result<Vec<(u32, String)>, Error> {
    let latest_download = get_latest_download_id(conn)?;
    if latest_download.is_none() {
        return Ok(Vec::new());
    }
    let download_id_filter = latest_download.unwrap();

    let results = teams::table
        .filter(teams::download_id.eq(download_id_filter))
        .select((teams::id, teams::name))
        .load::<(i32, String)>(conn)
        .map_err(|e| Error::Db(format!("Failed to load teams: {}", e)))?;

    Ok(results
        .into_iter()
        .map(|(id, name)| (id as u32, name))
        .collect())
}

pub fn get_players_for_team(
    conn: &mut SqliteConnection,
    team_id_in: u32,
) -> Result<Vec<crate::chpp::model::Player>, Error> {
    let latest_download = get_latest_download_id(conn)?;
    if latest_download.is_none() {
        return Ok(Vec::new());
    }
    let download_id_filter = latest_download.unwrap();

    let results: Vec<(PlayerEntity, Option<String>)> = players::table
        .left_join(countries::table.on(players::country_id.eq(countries::id)))
        .filter(players::team_id.eq(team_id_in as i32))
        .filter(players::download_id.eq(download_id_filter))
        .select((players::all_columns, countries::flag.nullable()))
        .load::<(PlayerEntity, Option<String>)>(conn)
        .map_err(|e| Error::Db(format!("Failed to load players: {}", e)))?;

    let mut players = Vec::new();
    for (entity, flag) in results {
        players.push(crate::chpp::model::Player {
            PlayerID: entity.id as u32,
            FirstName: entity.first_name,
            LastName: entity.last_name,
            PlayerNumber: if entity.player_number == 100 {
                None
            } else {
                Some(entity.player_number as u32)
            },
            Age: entity.age as u32,
            AgeDays: entity.age_days.map(|v| v as u32),
            TSI: entity.tsi as u32,
            PlayerForm: entity.player_form as u32,
            Statement: entity.statement,
            Experience: entity.experience as u32,
            Loyalty: entity.loyalty as u32,
            MotherClubBonus: entity.mother_club_bonus,
            Leadership: entity.leadership as u32,
            Salary: entity.salary as u32,
            IsAbroad: entity.is_abroad,
            Agreeability: entity.agreeability as u32,
            Aggressiveness: entity.aggressiveness as u32,
            Honesty: entity.honesty as u32,
            LeagueGoals: entity.league_goals.map(|v| v as u32),
            CupGoals: entity.cup_goals.map(|v| v as u32),
            FriendliesGoals: entity.friendlies_goals.map(|v| v as u32),
            CareerGoals: entity.career_goals.map(|v| v as u32),
            CareerHattricks: entity.career_hattricks.map(|v| v as u32),
            Speciality: entity.speciality.map(|v| v as u32),
            TransferListed: entity.transfer_listed,
            NationalTeamID: entity.national_team_id.map(|v| v as u32),
            CountryID: Some(entity.country_id as u32),
            Caps: entity.caps.map(|v| v as u32),
            CapsU20: entity.caps_u20.map(|v| v as u32),
            Cards: entity.cards.map(|v| v as u32),
            InjuryLevel: entity.injury_level.map(|v| v as i32),
            Sticker: entity.sticker,
            Flag: flag,
            ReferencePlayerID: None,
            PlayerSkills: if entity.stamina_skill.is_some() {
                Some(crate::chpp::model::PlayerSkills {
                    StaminaSkill: entity.stamina_skill.unwrap_or(0) as u32,
                    KeeperSkill: entity.keeper_skill.unwrap_or(0) as u32,
                    PlaymakerSkill: entity.playmaker_skill.unwrap_or(0) as u32,
                    ScorerSkill: entity.scorer_skill.unwrap_or(0) as u32,
                    PassingSkill: entity.passing_skill.unwrap_or(0) as u32,
                    WingerSkill: entity.winger_skill.unwrap_or(0) as u32,
                    DefenderSkill: entity.defender_skill.unwrap_or(0) as u32,
                    SetPiecesSkill: entity.set_pieces_skill.unwrap_or(0) as u32,
                })
            } else {
                None
            },
            LastMatch: if entity.last_match_date.is_some() {
                Some(crate::chpp::model::LastMatch {
                    Date: entity.last_match_date.unwrap_or_default(),
                    MatchId: entity.last_match_id.unwrap_or(0) as u32,
                    PositionCode: entity.last_match_position_code.unwrap_or(0) as u32,
                    PlayedMinutes: entity.last_match_played_minutes.unwrap_or(0) as u32,
                    Rating: entity.last_match_rating.map(|v| v as u32),
                    RatingEndOfMatch: entity.last_match_rating_end_of_match.map(|v| v as u32),
                })
            } else {
                None
            },
        });
    }

    Ok(players)
}

pub fn get_team(conn: &mut SqliteConnection, team_id: u32) -> Result<Option<Team>, Error> {
    use crate::db::schema::teams::dsl::*;

    let latest_download = get_latest_download_id(conn)?;
    if latest_download.is_none() {
        return Ok(None);
    }
    let download_id_filter = latest_download.unwrap();

    let result = teams
        .filter(id.eq(team_id as i32))
        .filter(download_id.eq(download_id_filter))
        .first::<TeamEntity>(conn)
        .optional()
        .map_err(|e| Error::Io(format!("Database error: {}", e)))?;

    match result {
        Some(entity) => {
            let team: Team = serde_json::from_str(&entity.raw_data).map_err(|e| {
                Error::Parse(format!("Failed to deserialize team data from DB: {}", e))
            })?;
            Ok(Some(team))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chpp::model::Language;
    use diesel::sqlite::SqliteConnection;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    fn establish_connection() -> SqliteConnection {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("Error connecting to :memory: database");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Error running migrations");
        conn
    }

    #[test]
    fn test_save_and_get_team() {
        let mut conn = establish_connection();

        let user = User {
            UserID: 12345,
            Name: "Test User".to_string(),
            Loginname: "testuser".to_string(),
            SupporterTier: SupporterTier::Gold,
            SignupDate: "2000-01-01".to_string(),
            ActivationDate: "2000-01-02".to_string(),
            LastLoginDate: "2023-01-01".to_string(),
            HasManagerLicense: true,
            Language: Language {
                LanguageID: 1,
                LanguageName: "English".to_string(),
            },
        };

        let team_xml = r#"
            <Team>
                <TeamID>99999</TeamID>
                <TeamName>Persistence Test Team</TeamName>
                <ShortTeamName>PTT</ShortTeamName>
                <IsPrimaryClub>True</IsPrimaryClub>
                <FoundedDate>2000-01-01</FoundedDate>
                <IsDeactivated>False</IsDeactivated>
                <Arena>
                    <ArenaID>1</ArenaID>
                    <ArenaName>Arena</ArenaName>
                </Arena>
                <Country>
                    <CountryID>1</CountryID>
                    <CountryName>Country</CountryName>
                </Country>
                <Region>
                    <RegionID>100</RegionID>
                    <RegionName>Test Region</RegionName>
                </Region>
                <Trainer>
                    <PlayerID>888</PlayerID>
                </Trainer>
                <PowerRating>
                    <GlobalRanking>10</GlobalRanking>
                    <LeagueRanking>5</LeagueRanking>
                    <RegionRanking>2</RegionRanking>
                    <PowerRating>1500</PowerRating>
                </PowerRating>
            </Team>
        "#;
        let team: Team = serde_xml_rs::from_str(team_xml).expect("Failed to parse team XML");

        // Create a download record
        use crate::db::schema::downloads;
        use chrono::Utc;
        let download_entity = DownloadEntity {
            id: 0,
            timestamp: Utc::now().to_rfc3339(),
            status: "completed".to_string(),
        };
        diesel::insert_into(downloads::table)
            .values(&download_entity)
            .execute(&mut conn)
            .expect("Failed to create download");

        save_team(&mut conn, &team, &user, 0).expect("Failed to save team");

        let saved_team = get_team(&mut conn, 99999)
            .expect("Failed to get team")
            .unwrap();
        assert_eq!(saved_team.TeamID, "99999");
        assert_eq!(saved_team.TeamName, "Persistence Test Team");
        assert_eq!(saved_team.ShortTeamName, Some("PTT".to_string()));
        assert_eq!(saved_team.Trainer.unwrap().PlayerID, 888);
        assert_eq!(saved_team.PowerRating.unwrap().PowerRating, 1500);
    }

    #[test]
    fn test_full_persistence_hierarchy() {
        let mut conn = establish_connection();

        let user = User {
            UserID: 200,
            Name: "Hierarchical User".to_string(),
            Loginname: "hierarchy".to_string(),
            SupporterTier: SupporterTier::Platinum,
            SignupDate: "2020-01-01".to_string(),
            ActivationDate: "2020-01-02".to_string(),
            LastLoginDate: "2023-01-01".to_string(),
            HasManagerLicense: true,
            Language: Language {
                LanguageID: 2,
                LanguageName: "Swedish".to_string(),
            },
        };

        // Currency for Country
        let currency = Currency {
            CurrencyID: 5,
            CurrencyName: "Swedish Krona".to_string(),
            Rate: Some(1.0),
            Symbol: Some("kr".to_string()),
        };

        let team_xml = r#"
            <Team>
                <TeamID>88888</TeamID>
                <TeamName>Hierarchy Team</TeamName>
                <ShortTeamName>HT</ShortTeamName>
                <IsPrimaryClub>True</IsPrimaryClub>
                <FoundedDate>2010-01-01</FoundedDate>
                <IsDeactivated>False</IsDeactivated>
                <Arena>
                    <ArenaID>200</ArenaID>
                    <ArenaName>Hierarchy Arena</ArenaName>
                </Arena>
                <Country>
                    <CountryID>100</CountryID>
                    <CountryName>Sweden</CountryName>
                </Country>
                <Region>
                    <RegionID>1001</RegionID>
                    <RegionName>Stockholm</RegionName>
                </Region>
                 <League>
                    <LeagueID>1000</LeagueID>
                    <LeagueName>Allsvenskan</LeagueName>
                </League>
                <Trainer>
                    <PlayerID>999</PlayerID>
                </Trainer>
            </Team>
        "#;
        let mut team: Team = serde_xml_rs::from_str(team_xml).expect("Failed to parse team XML");

        team.Country = Some(Country {
            CountryID: 100,
            CountryName: "Sweden".to_string(),
            Currency: Some(currency),
            CountryCode: None,
            DateFormat: None,
            TimeFormat: None,
        });

        save_team(&mut conn, &team, &user, 0).expect("Failed to save team");

        use crate::db::schema::languages::dsl::*;
        let langs = languages
            .filter(crate::db::schema::languages::id.eq(2))
            .load::<LanguageEntity>(&mut conn)
            .expect("Error loading language");
        assert_eq!(langs.len(), 1);
        assert_eq!(langs[0].name, "Swedish");

        use crate::db::schema::currencies::dsl::*;
        let currs = currencies
            .filter(crate::db::schema::currencies::id.eq(5))
            .load::<CurrencyEntity>(&mut conn)
            .expect("Error loading currency");
        assert_eq!(currs.len(), 1);
        assert_eq!(currs[0].name, "Swedish Krona");
        assert_eq!(currs[0].rate, Some(1.0));
        assert_eq!(currs[0].symbol, Some("kr".to_string()));

        use crate::db::schema::countries::dsl::*;
        let cnts = countries
            .filter(crate::db::schema::countries::id.eq(100))
            .load::<CountryEntity>(&mut conn)
            .expect("Error loading country");
        assert_eq!(cnts.len(), 1);
        assert_eq!(cnts[0].name, "Sweden");
        assert_eq!(cnts[0].currency_id, Some(5)); // Foreign Key verification

        use crate::db::schema::users::dsl::*;
        let usrs = users
            .filter(crate::db::schema::users::id.eq(200))
            .load::<UserEntity>(&mut conn)
            .expect("Error loading user");
        assert_eq!(usrs.len(), 1);
        assert_eq!(usrs[0].language_id, Some(2)); // Foreign Key verification

        use crate::db::schema::teams::dsl::*;
        let tms = teams
            .filter(crate::db::schema::teams::id.eq(88888))
            .load::<TeamEntity>(&mut conn)
            .expect("Error loading team");
        assert_eq!(tms.len(), 1);
        assert_eq!(tms[0].user_id, Some(200));
        assert_eq!(tms[0].country_id, Some(100));
        assert_eq!(tms[0].region_id, Some(1001));
        assert_eq!(tms[0].league_id, Some(1000));
    }

    #[test]
    fn test_flag_emoji() {
        assert_eq!(Some("🇸🇪".to_string()), get_flag_emoji(Some("SE")));
        assert_eq!(Some("🇮🇪".to_string()), get_flag_emoji(Some("IE")));
        assert_eq!(Some("🇫🇷".to_string()), get_flag_emoji(Some("FR")));
        assert_eq!(Some("🇬🇱".to_string()), get_flag_emoji(Some("GL")));
        // The way the flag displays below (for invalid pair) depends on the system (and probably the font?)
        assert_eq!(Some("🇧🇨".to_string()), get_flag_emoji(Some("BC")));
    }
}
