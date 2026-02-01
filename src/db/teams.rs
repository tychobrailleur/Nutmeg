use crate::chpp::error::Error;
use crate::chpp::model::{
    Country, Cup, Currency, Language, League, Region, SupporterTier, Team, User,
};
use crate::db::schema::{countries, cups, currencies, languages, leagues, regions, teams, users};
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
#[diesel(table_name = teams)]
struct TeamEntity {
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

    let entity = CountryEntity {
        id: country.CountryID as i32,
        name: country.CountryName.clone(),
        currency_id: country.Currency.as_ref().map(|c| c.CurrencyID as i32),
    };
    diesel::insert_into(countries::table)
        .values(&entity)
        .on_conflict(countries::id)
        .do_update()
        .set((
            countries::name.eq(&entity.name),
            countries::currency_id.eq(&entity.currency_id),
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
    };
    diesel::insert_into(leagues::table)
        .values(&entity)
        .on_conflict(leagues::id)
        .do_update()
        .set((
            leagues::name.eq(&entity.name),
            leagues::country_id.eq(&entity.country_id),
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
pub fn save_team(conn: &mut SqliteConnection, team: &Team, user: &User) -> Result<(), Error> {
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
        cup_still_in: team.Cup.as_ref().map(|c| c.StillInCup),
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
        .on_conflict(teams::id)
        .do_update()
        .set((
            teams::user_id.eq(&entity.user_id),
            teams::name.eq(&entity.name),
            teams::raw_data.eq(&entity.raw_data),
            teams::short_name.eq(&entity.short_name),
            teams::is_primary_club.eq(&entity.is_primary_club),
            teams::founded_date.eq(&entity.founded_date),
            teams::arena_id.eq(&entity.arena_id),
            teams::arena_name.eq(&entity.arena_name),
            teams::league_id.eq(&entity.league_id),
            teams::league_name.eq(&entity.league_name),
            teams::country_id.eq(&entity.country_id),
            teams::country_name.eq(&entity.country_name),
            teams::region_id.eq(&entity.region_id),
            teams::region_name.eq(&entity.region_name),
            teams::homepage.eq(&entity.homepage),
            teams::dress_uri.eq(&entity.dress_uri),
            teams::dress_alternate_uri.eq(&entity.dress_alternate_uri),
            teams::logo_url.eq(&entity.logo_url),
            teams::trainer_id.eq(&entity.trainer_id),
            teams::cup_still_in.eq(&entity.cup_still_in),
            teams::cup_id.eq(&entity.cup_id),
            teams::cup_name.eq(&entity.cup_name),
            teams::cup_league_level.eq(&entity.cup_league_level),
            teams::cup_level.eq(&entity.cup_level),
            teams::cup_level_index.eq(&entity.cup_level_index),
            teams::cup_match_round.eq(&entity.cup_match_round),
            teams::cup_match_rounds_left.eq(&entity.cup_match_rounds_left),
            teams::power_rating_global.eq(&entity.power_rating_global),
            teams::power_rating_league.eq(&entity.power_rating_league),
            teams::power_rating_region.eq(&entity.power_rating_region),
            teams::power_rating_indiv.eq(&entity.power_rating_indiv),
            teams::friendly_team_id.eq(&entity.friendly_team_id),
            teams::league_level_unit_id.eq(&entity.league_level_unit_id),
            teams::league_level_unit_name.eq(&entity.league_level_unit_name),
            teams::league_level.eq(&entity.league_level),
            teams::number_of_victories.eq(&entity.number_of_victories),
            teams::number_of_undefeated.eq(&entity.number_of_undefeated),
            teams::number_of_visits.eq(&entity.number_of_visits),
            teams::team_rank.eq(&entity.team_rank),
            teams::fanclub_id.eq(&entity.fanclub_id),
            teams::fanclub_name.eq(&entity.fanclub_name),
            teams::fanclub_size.eq(&entity.fanclub_size),
            teams::color_background.eq(&entity.color_background),
            teams::color_primary.eq(&entity.color_primary),
            teams::is_bot.eq(&entity.is_bot),
            teams::bot_since.eq(&entity.bot_since),
            teams::youth_team_id.eq(&entity.youth_team_id),
            teams::youth_team_name.eq(&entity.youth_team_name),
        ))
        .execute(conn)
        .map_err(|e| Error::Io(format!("Database error: {}", e)))?;

    Ok(())
}

pub fn get_team(conn: &mut SqliteConnection, team_id: u32) -> Result<Option<Team>, Error> {
    use crate::db::schema::teams::dsl::*;

    let result = teams
        .find(team_id as i32)
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

        save_team(&mut conn, &team, &user).expect("Failed to save team");

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
        });

        save_team(&mut conn, &team, &user).expect("Failed to save team");

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
}
