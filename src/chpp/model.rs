/* model.rs
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

use serde::{Deserialize, Serialize};
//use uuid::Uuid;

// Utility function for deserialisation
fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    match s.trim().to_lowercase().as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" | "" => Ok(false),
        _ => Err(serde::de::Error::custom(format!(
            "Expected True/False/1/0, got '{}'",
            s
        ))),
    }
}

fn deserialize_option_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Deserialize::deserialize(deserializer)?;
    match s {
        Some(s) => match s.trim().to_lowercase().as_str() {
            "true" | "1" => Ok(Some(true)),
            "false" | "0" | "" => Ok(Some(false)),
            _ => Ok(None),
        },
        None => Ok(None),
    }
}

fn deserialize_empty_tag_is_none<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s.as_deref() {
        None | Some("") => Ok(None),
        Some(v) => v.parse().map(Some).map_err(serde::de::Error::custom),
    }
}

fn deserialize_player_number<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s.as_deref() {
        None | Some("100") => Ok(None), // If player number is 100, the player has no number.
        Some(v) => v.parse().map(Some).map_err(serde::de::Error::custom),
    }
}

fn serialize_bool<S>(x: &bool, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(if *x { "true" } else { "false" })
}

fn serialize_option_bool<S>(x: &Option<bool>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match x {
        Some(b) => s.serialize_str(if *b { "true" } else { "false" }),
        None => s.serialize_none(),
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug, PartialEq, Clone, Copy)]
pub enum SupporterTier {
    None,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

impl<'de> Deserialize<'de> for SupporterTier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "none" => Ok(SupporterTier::None),
            "silver" => Ok(SupporterTier::Silver),
            "gold" => Ok(SupporterTier::Gold),
            "platinum" => Ok(SupporterTier::Platinum),
            "diamond" => Ok(SupporterTier::Diamond),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown SupporterTier: {}",
                s
            ))),
        }
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Language {
    pub LanguageID: u32,
    pub LanguageName: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    #[allow(dead_code)]
    pub UserID: u32,
    #[allow(dead_code)]
    pub Language: Language,
    pub Name: String,
    pub Loginname: String,
    pub SupporterTier: SupporterTier,
    pub SignupDate: String,
    pub ActivationDate: String,
    pub LastLoginDate: String,
    #[serde(deserialize_with = "deserialize_bool")]
    pub HasManagerLicense: bool,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Arena {
    pub ArenaID: u32,
    pub ArenaName: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct League {
    pub LeagueID: u32,
    pub LeagueName: String,
    pub ShortName: Option<String>,
    pub Continent: Option<String>,
    pub Season: Option<u32>,
    pub SeasonOffset: Option<i32>,
    pub MatchRound: Option<u32>,
    pub ZoneName: Option<String>,
    pub EnglishName: Option<String>,
    pub LanguageID: Option<u32>,
    pub NationalTeamId: Option<u32>,
    pub U20TeamId: Option<u32>,
    pub ActiveTeams: Option<u32>,
    pub ActiveUsers: Option<u32>,
    pub NumberOfLevels: Option<u32>,
    pub LeagueSystemId: Option<u32>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Country {
    pub CountryID: u32,
    pub CountryName: String,
    pub Currency: Option<Currency>,
    pub CountryCode: Option<String>,
    pub DateFormat: Option<String>,
    pub TimeFormat: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Currency {
    pub CurrencyID: u32,
    pub CurrencyName: String,
    pub Rate: Option<f64>, // Relative to SEK
    pub Symbol: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Region {
    pub RegionID: u32,
    pub RegionName: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Trainer {
    pub PlayerID: u32,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Fanclub {
    pub FanclubID: u32,
    pub FanclubName: String,
    pub FanclubSize: u32,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Cup {
    #[serde(
        deserialize_with = "deserialize_option_bool",
        serialize_with = "serialize_option_bool",
        default
    )]
    pub StillInCup: Option<bool>,
    pub CupID: Option<u32>,
    pub CupName: Option<String>,
    pub CupLeagueLevel: Option<u32>, // 0 = National (LeagueLevel 1-6), 7-9 = Divisional.
    pub CupLevel: Option<u32>,       // 1 = National/Divisional, 2 = Challenger, 3 = Consolation.
    pub CupLevelIndex: Option<u32>, // Always 1 for National and Consolation cups, for Challenger cup: 1 = Emerald, 2 = Ruby, 3 = Sapphire
    pub MatchRound: Option<u32>,
    pub MatchRoundsLeft: Option<u32>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LeagueLevelUnit {
    pub LeagueLevelUnitID: u32,
    pub LeagueLevelUnitName: String,
    pub LeagueLevel: u32,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PowerRating {
    pub GlobalRanking: u32,
    pub LeagueRanking: u32,
    pub RegionRanking: u32,
    pub PowerRating: u32,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TeamColors {
    pub BackgroundColor: String,
    pub Color: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BotStatus {
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub IsBot: bool,
    pub BotSince: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Trophy {
    pub TrophyTypeId: Option<u32>,
    pub TrophySeason: Option<u32>,
    pub LeagueLevel: Option<u32>,
    pub LeagueLevelUnitId: Option<String>,
    pub LeagueLevelUnitName: Option<String>,
    pub GainedDate: Option<String>,
    pub ImageUrl: Option<String>,
    pub CupLeagueLevel: Option<u32>,
    pub CupLevel: Option<u32>,
    pub CupLevelIndex: Option<u32>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TrophyListWrapper {
    #[serde(rename = "Trophy", default)]
    pub trophies: Vec<Trophy>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlayerSkills {
    pub StaminaSkill: u32,
    pub KeeperSkill: u32,
    pub PlaymakerSkill: u32,
    pub ScorerSkill: u32,
    pub PassingSkill: u32,
    pub WingerSkill: u32,
    pub DefenderSkill: u32,
    pub SetPiecesSkill: u32,
}

// TODO Check whether this can be Match instead of LastMatch...
#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LastMatch {
    pub Date: String,
    pub MatchId: u32,
    pub PositionCode: u32,
    pub PlayedMinutes: u32,
    pub Rating: Option<f64>,
    pub RatingEndOfMatch: Option<f64>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
// Player maps to Player in players and playerdetails
pub struct Player {
    pub PlayerID: u32,
    pub FirstName: String,
    pub LastName: String,
    pub NickName: Option<String>,
    #[serde(deserialize_with = "deserialize_player_number")]
    pub PlayerNumber: Option<u32>,
    pub Age: u32,
    pub AgeDays: Option<u32>,
    pub TSI: u32,
    pub PlayerForm: u32,
    pub Statement: Option<String>,
    pub Experience: u32,
    pub Loyalty: u32,
    pub ReferencePlayerID: Option<u32>,
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub MotherClubBonus: bool,
    pub Leadership: u32,
    pub Salary: u32,
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub IsAbroad: bool,
    pub Agreeability: u32,
    pub Aggressiveness: u32,
    pub Honesty: u32,
    pub LeagueGoals: Option<u32>,
    pub CupGoals: Option<u32>,
    pub FriendliesGoals: Option<u32>,
    pub CareerGoals: Option<u32>,
    pub CareerHattricks: Option<u32>,

    pub CareerAssists: Option<u32>,
    pub Speciality: Option<u32>,
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub TransferListed: bool,
    pub NationalTeamID: Option<u32>,
    pub CountryID: Option<u32>,
    pub Caps: Option<u32>,
    pub CapsU20: Option<u32>,
    pub Cards: Option<u32>,
    pub InjuryLevel: Option<i32>, // -1 = No injury, 0 = Bruised, >0 = Weeks
    pub Sticker: Option<String>,
    #[serde(skip)]
    pub AvatarBlob: Option<Vec<u8>>,
    #[serde(skip)]
    pub Flag: Option<String>,
    pub PlayerSkills: Option<PlayerSkills>, // Only visible for own team or if authorized
    pub ArrivalDate: Option<String>,
    pub PlayerCategoryId: Option<u32>, // 1 = keeper, 2 wingbacl, 3 central defender, 4 winger,
    // 5 inner midfield, 6 forward, 7 sub, 8 reserve, 9 extra 1, 10 extra 2, 0 no category
    pub MotherClub: Option<MotherClub>,
    pub NativeCountryID: Option<u32>,
    pub NativeLeagueID: Option<u32>,
    pub NativeLeagueName: Option<String>,
    pub MatchesCurrentTeam: Option<u32>,
    pub GoalsCurrentTeam: Option<u32>,
    pub AssistsCurrentTeam: Option<u32>,
    pub LastMatch: Option<LastMatch>,
    #[serde(default, deserialize_with = "deserialize_empty_tag_is_none")]
    pub GenderID: Option<u32>,
}

impl Player {
    /// Merges two players, typically one from the basic players endpoint
    ///    and one from the detailed playerdetails endpoint.
    ///
    /// Strategy:
    /// - If detailed data is available, use it as the primary source
    /// - Fill in any None fields in detailed data with values from basic data
    /// - This ensures we capture all available information from both endpoints
    ///
    /// Note: PlayerSkills are only available in playerdetails for own team,
    /// so basic data will never have skills to contribute.
    pub fn merge_player_data(
        &self,
        other: Option<crate::chpp::model::Player>,
    ) -> crate::chpp::model::Player {
        match other {
            Some(mut o) => {
                if o.PlayerNumber.is_none() && self.PlayerNumber.is_some() {
                    o.PlayerNumber = self.PlayerNumber;
                }
                if o.AgeDays.is_none() && self.AgeDays.is_some() {
                    o.AgeDays = self.AgeDays;
                }
                if o.Statement.is_none() && self.Statement.is_some() {
                    o.Statement = self.Statement.clone();
                }
                if o.ReferencePlayerID.is_none() && self.ReferencePlayerID.is_some() {
                    o.ReferencePlayerID = self.ReferencePlayerID;
                }
                if o.LeagueGoals.is_none() && self.LeagueGoals.is_some() {
                    o.LeagueGoals = self.LeagueGoals;
                }
                if o.CupGoals.is_none() && self.CupGoals.is_some() {
                    o.CupGoals = self.CupGoals;
                }
                if o.FriendliesGoals.is_none() && self.FriendliesGoals.is_some() {
                    o.FriendliesGoals = self.FriendliesGoals;
                }
                if o.CareerGoals.is_none() && self.CareerGoals.is_some() {
                    o.CareerGoals = self.CareerGoals;
                }
                if o.CareerHattricks.is_none() && self.CareerHattricks.is_some() {
                    o.CareerHattricks = self.CareerHattricks;
                }
                if o.Speciality.is_none() && self.Speciality.is_some() {
                    o.Speciality = self.Speciality;
                }
                if o.NationalTeamID.is_none() && self.NationalTeamID.is_some() {
                    o.NationalTeamID = self.NationalTeamID;
                }
                if o.CountryID.is_none() && self.CountryID.is_some() {
                    o.CountryID = self.CountryID;
                }
                // Set country ID to native country ID if country ID is not present.
                if o.CountryID.is_none() && o.NativeCountryID.is_some() {
                    o.CountryID = o.NativeCountryID;
                }
                // National team stats
                if o.Caps.is_none() && self.Caps.is_some() {
                    o.Caps = self.Caps;
                }
                if o.CapsU20.is_none() && self.CapsU20.is_some() {
                    o.CapsU20 = self.CapsU20;
                }
                if o.Cards.is_none() && self.Cards.is_some() {
                    o.Cards = self.Cards;
                }
                if o.InjuryLevel.is_none() && self.InjuryLevel.is_some() {
                    o.InjuryLevel = self.InjuryLevel;
                }
                if o.Sticker.is_none() && self.Sticker.is_some() {
                    o.Sticker = self.Sticker.clone();
                }
                if o.LastMatch.is_none() && self.LastMatch.is_some() {
                    o.LastMatch = self.LastMatch.clone();
                }
                if o.ArrivalDate.is_none() && self.ArrivalDate.is_some() {
                    o.ArrivalDate = self.ArrivalDate.clone();
                }
                if o.PlayerCategoryId.is_none() && self.PlayerCategoryId.is_some() {
                    o.PlayerCategoryId = self.PlayerCategoryId;
                }
                if o.MotherClub.is_none() && self.MotherClub.is_some() {
                    o.MotherClub = self.MotherClub.clone();
                }
                if o.NativeCountryID.is_none() && self.NativeCountryID.is_some() {
                    o.NativeCountryID = self.NativeCountryID;
                }
                if o.NativeLeagueID.is_none() && self.NativeLeagueID.is_some() {
                    o.NativeLeagueID = self.NativeLeagueID;
                }
                if o.NativeLeagueName.is_none() && self.NativeLeagueName.is_some() {
                    o.NativeLeagueName = self.NativeLeagueName.clone();
                }
                if o.MatchesCurrentTeam.is_none() && self.MatchesCurrentTeam.is_some() {
                    o.MatchesCurrentTeam = self.MatchesCurrentTeam;
                }
                if o.GoalsCurrentTeam.is_none() && self.GoalsCurrentTeam.is_some() {
                    o.GoalsCurrentTeam = self.GoalsCurrentTeam;
                }
                if o.AssistsCurrentTeam.is_none() && self.AssistsCurrentTeam.is_some() {
                    o.AssistsCurrentTeam = self.AssistsCurrentTeam;
                }
                if o.CareerAssists.is_none() && self.CareerAssists.is_some() {
                    o.CareerAssists = self.CareerAssists;
                }
                if o.GenderID.is_none() && self.GenderID.is_some() {
                    o.GenderID = self.GenderID;
                }

                o
            }
            None => {
                // Other struct missing, just return self
                self.clone()
            }
        }
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MotherClub {
    pub TeamID: u32,
    pub TeamName: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlayerList {
    #[serde(rename = "Player")]
    pub players: Vec<Player>,
}

// Team Details:
// https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=teamdetails

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Team {
    pub TeamID: String,
    pub TeamName: String,
    pub ShortTeamName: Option<String>,
    #[serde(
        deserialize_with = "deserialize_option_bool",
        serialize_with = "serialize_option_bool",
        default
    )]
    pub IsPrimaryClub: Option<bool>,
    pub FoundedDate: Option<String>,
    #[serde(
        deserialize_with = "deserialize_option_bool",
        serialize_with = "serialize_option_bool",
        default
    )]
    pub IsDeactivated: Option<bool>,
    pub Arena: Option<Arena>,
    pub League: Option<League>,
    pub Country: Option<Country>,
    pub Region: Option<Region>,
    pub Trainer: Option<Trainer>,
    pub HomePage: Option<String>,
    pub Cup: Option<Cup>,
    pub PowerRating: Option<PowerRating>,
    #[serde(default, deserialize_with = "deserialize_empty_tag_is_none")]
    // Empty tag <FriendlyTeamID /> seems to fail for Option<u32>
    // so use a custom deserializer for these fields.
    pub FriendlyTeamID: Option<u32>,
    pub LeagueLevelUnit: Option<LeagueLevelUnit>,
    #[serde(default, deserialize_with = "deserialize_empty_tag_is_none")]
    pub NumberOfVictories: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_empty_tag_is_none")]
    pub NumberOfUndefeated: Option<u32>,
    pub Fanclub: Option<Fanclub>,
    pub LogoURL: Option<String>,
    pub TeamColors: Option<TeamColors>,
    pub DressURI: Option<String>,
    pub DressAlternateURI: Option<String>,
    pub BotStatus: Option<BotStatus>,
    #[serde(default, deserialize_with = "deserialize_empty_tag_is_none")]
    pub TeamRank: Option<u32>,
    pub YouthTeamID: Option<u32>,
    pub YouthTeamName: Option<String>,
    pub NumberOfVisits: Option<u32>,
    // #[serde(rename = "TrophyList")]
    // pub TrophyList: Option<TrophyListWrapper>,
    pub PlayerList: Option<PlayerList>,
    #[serde(deserialize_with = "deserialize_option_bool", default)]
    pub PossibleToChallengeMidweek: Option<bool>,
    #[serde(deserialize_with = "deserialize_option_bool", default)]
    pub PossibleToChallengeWeekend: Option<bool>,
    // TODO: Verify if GenderID is actually returned by teamdetails.
    // If not, we might need to infer it or keep it as Option.
    // Assuming it might be there or we default to 1 in DB.
    pub GenderID: Option<u32>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct Teams {
    #[serde(rename = "Team")]
    pub Teams: Vec<Team>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct HattrickData {
    pub Teams: Teams,
    #[allow(dead_code)]
    pub User: User,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename = "HattrickData")]
pub struct PlayersData {
    pub Team: Team,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct PlayerDetailsData {
    pub Player: Player,
}

#[allow(non_snake_case)]
#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct WorldCountry {
    pub CountryID: Option<u32>,
    pub CountryName: Option<String>,
    pub CurrencyName: Option<String>,
    pub CurrencyRate: Option<String>, // Stores comma-separated floats
    pub CountryCode: Option<String>,
    pub DateFormat: Option<String>,
    pub TimeFormat: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct WorldLeague {
    pub LeagueID: u32,
    pub LeagueName: String,
    pub Country: WorldCountry,
    pub Season: Option<u32>,
    pub SeasonOffset: Option<i32>,
    pub MatchRound: Option<u32>,
    pub ShortName: Option<String>,
    pub Continent: Option<String>,
    pub ZoneName: Option<String>,
    pub EnglishName: Option<String>,
    pub LanguageId: Option<u32>, // Watch out, lowercase D!
    pub LanguageName: Option<String>,
    pub NationalTeamId: Option<u32>,
    pub U20TeamId: Option<u32>,
    pub ActiveTeams: Option<u32>,
    pub ActiveUsers: Option<u32>,
    pub NumberOfLevels: Option<u32>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct WorldLeagueList {
    #[serde(rename = "League")]
    pub Leagues: Vec<WorldLeague>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct WorldDetails {
    pub LeagueList: WorldLeagueList,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChppErrorResponse {
    pub Error: String,
    pub ErrorCode: u32,
    pub ErrorGUID: Option<String>,
    pub Request: Option<String>,
    pub LineNumber: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_xml_rs::from_str;

    #[test]
    fn test_deserialize_bool_true() {
        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct BooleanWrapper {
            #[serde(deserialize_with = "deserialize_bool")]
            val: bool,
        }

        let xml = "<BooleanWrapper><val>True</val></BooleanWrapper>";
        let res: BooleanWrapper = from_str(xml).unwrap();
        assert_eq!(res.val, true);

        let xml = "<BooleanWrapper><val>true</val></BooleanWrapper>";
        let res: BooleanWrapper = from_str(xml).unwrap();
        assert_eq!(res.val, true);
    }

    #[test]
    fn test_deserialize_bool_false() {
        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct BooleanWrapper {
            #[serde(deserialize_with = "deserialize_bool")]
            val: bool,
        }

        let xml = "<BooleanWrapper><val>False</val></BooleanWrapper>";
        let res: BooleanWrapper = from_str(xml).unwrap();
        assert_eq!(res.val, false);
    }

    #[test]
    fn test_world_country_currency_rate() {
        // WorldCountry deserialization test with comma float AND attribute (reproduce potential attribute issue)
        let xml = r#"<WorldCountry Available="True"><CountryID>1</CountryID><CountryName>Test</CountryName><CurrencyRate>2,5</CurrencyRate></WorldCountry>"#;
        let c: WorldCountry =
            from_str(xml).expect("Should deserialize WorldCountry with attribute");
        assert_eq!(c.CurrencyRate, Some("2,5".to_string()));

        // Test with dot float
        let xml2 = r#"<WorldCountry><CountryID>2</CountryID><CountryName>Test2</CountryName><CurrencyRate>2.5</CurrencyRate></WorldCountry>"#;
        let c2: WorldCountry = from_str(xml2).expect("Should deserialize WorldCountry");
        assert_eq!(c2.CurrencyRate, Some("2.5".to_string()));

        // Test with empty (should handle logic downstream, but parsing string handles it)
        let xml3 = r#"<WorldCountry><CountryID>3</CountryID><CountryName>Test3</CountryName><CurrencyRate></CurrencyRate></WorldCountry>"#;
        let c3: WorldCountry = from_str(xml3).expect("Should deserialize empty rate");
        assert_eq!(c3.CurrencyRate, Some("".to_string()));
    }

    #[test]
    fn test_world_country_unavailable() {
        let xml = r#"<WorldCountry Available="False" />"#;
        let c: WorldCountry = from_str(xml).expect("Should deserialize unavailable WorldCountry");
        assert_eq!(c.CountryID, None);
        assert_eq!(c.CountryName, None);
    }

    #[test]
    fn test_supporter_tier_case_insensitive() {
        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct TierWrapper {
            tier: SupporterTier,
        }

        let xml = "<TierWrapper><tier>gold</tier></TierWrapper>";
        let res: TierWrapper = from_str(xml).unwrap();
        assert_eq!(res.tier, SupporterTier::Gold);

        let xml = "<TierWrapper><tier>Silver</tier></TierWrapper>";
        let res: TierWrapper = from_str(xml).unwrap();
        assert_eq!(res.tier, SupporterTier::Silver);
    }

    // TODO in the following tests, anonymise the following XML outputs,
    // if not done already, and extract to test resources.
    #[test]
    fn test_deserialize_team() {
        let xml = r#"
        <HattrickData>
            <Teams>
                <Team>
                    <TeamID>1000</TeamID>
                    <TeamName>Test Team A</TeamName>
                    <ShortTeamName>Testers</ShortTeamName>
                    <IsPrimaryClub>True</IsPrimaryClub>
                    <FoundedDate>2019-10-24 20:20:00</FoundedDate>
                    <IsDeactivated>False</IsDeactivated>
                    <Arena>
                        <ArenaID>1000</ArenaID>
                        <ArenaName>Test Arena</ArenaName>
                    </Arena>
                    <League>
                        <LeagueID>1</LeagueID>
                        <LeagueName>Test League</LeagueName>
                    </League>
                    <Country>
                        <CountryID>1</CountryID>
                        <CountryName>Test Country</CountryName>
                    </Country>
                    <Region>
                        <RegionID>10</RegionID>
                        <RegionName>Test Region</RegionName>
                    </Region>
                    <Trainer>
                        <PlayerID>2000</PlayerID>
                    </Trainer>
                    <HomePage></HomePage>
                    <PowerRating>
                        <GlobalRanking>45701</GlobalRanking>
                        <LeagueRanking>148</LeagueRanking>
                        <RegionRanking>7</RegionRanking>
                        <PowerRating>936</PowerRating>
                    </PowerRating>
                    <FriendlyTeamID>0</FriendlyTeamID>
                    <LeagueLevelUnit>
                        <LeagueLevelUnitID>8000</LeagueLevelUnitID>
                        <LeagueLevelUnitName>IV.1</LeagueLevelUnitName>
                        <LeagueLevel>4</LeagueLevel>
                    </LeagueLevelUnit>
                    <NumberOfVictories>2</NumberOfVictories>
                    <NumberOfUndefeated>2</NumberOfUndefeated>
                    <Fanclub>
                        <FanclubID>3000</FanclubID>
                        <FanclubName>Test Ultras</FanclubName>
                        <FanclubSize>100</FanclubSize>
                    </Fanclub>
                    <LogoURL>//res.hattrick.org/teamlogo/0/0/0/0/0.png</LogoURL>
                    <TeamColors>
                        <BackgroundColor>288032</BackgroundColor>
                        <Color>ffffff</Color>
                    </TeamColors>
                    <DressURI>//res.hattrick.org/kits/0/0/0/0/matchKitSmall.png</DressURI>
                    <DressAlternateURI>//res.hattrick.org/kits/0/0/0/1/matchKitSmall.png</DressAlternateURI>
                    <BotStatus>
                        <IsBot>False</IsBot>
                    </BotStatus>
                    <TeamRank>1</TeamRank>
                    <YouthTeamID>0</YouthTeamID>
                    <YouthTeamName></YouthTeamName>
                    <NumberOfVisits>1</NumberOfVisits>
                    <PossibleToChallengeMidweek>False</PossibleToChallengeMidweek>
                    <PossibleToChallengeWeekend>False</PossibleToChallengeWeekend>
                </Team>
            </Teams>
            <User>
               <UserID>1</UserID>
               <Language>
                 <LanguageID>1</LanguageID>
                 <LanguageName>English</LanguageName>
               </Language>
               <Name>Test User</Name>
               <Loginname>TestUser</Loginname>
               <SupporterTier>None</SupporterTier>
               <SignupDate>Date</SignupDate>
               <ActivationDate>Date</ActivationDate>
               <LastLoginDate>Date</LastLoginDate>
               <HasManagerLicense>True</HasManagerLicense>
            </User>
        </HattrickData>
        "#;

        let res: HattrickData = from_str(xml).expect("Failed to deserialize team XML");
        let team = &res.Teams.Teams[0];

        assert_eq!(team.TeamID, "1000");
        assert_eq!(team.TeamName, "Test Team A");
        assert_eq!(team.IsPrimaryClub, Some(true));
        assert_eq!(team.IsDeactivated, Some(false));
    }
    #[test]
    fn test_deserialize_team_without_colors() {
        let xml = r#"
        <HattrickData>
            <Teams>
                <Team>
                    <TeamID>2000</TeamID>
                    <TeamName>Test Team B</TeamName>
                    <ShortTeamName>Testers B</ShortTeamName>
                    <IsPrimaryClub>True</IsPrimaryClub>
                    <FoundedDate>2023-02-17 20:58:00</FoundedDate>
                    <IsDeactivated>False</IsDeactivated>
                    <Arena>
                        <ArenaID>2000</ArenaID>
                        <ArenaName>Test Arena B</ArenaName>
                    </Arena>
                    <League>
                        <LeagueID>2</LeagueID>
                        <LeagueName>Test League B</LeagueName>
                    </League>
                    <Country>
                        <CountryID>2</CountryID>
                        <CountryName>Test Country B</CountryName>
                    </Country>
                    <Region>
                        <RegionID>20</RegionID>
                        <RegionName>Test Region B</RegionName>
                    </Region>
                    <Trainer>
                        <PlayerID>3000</PlayerID>
                    </Trainer>
                    <HomePage></HomePage>
                    <PowerRating>
                        <GlobalRanking>1000</GlobalRanking>
                        <LeagueRanking>100</LeagueRanking>
                        <RegionRanking>10</RegionRanking>
                        <PowerRating>500</PowerRating>
                    </PowerRating>
                    <FriendlyTeamID>0</FriendlyTeamID>
                    <LeagueLevelUnit>
                        <LeagueLevelUnitID>9000</LeagueLevelUnitID>
                        <LeagueLevelUnitName>V.1</LeagueLevelUnitName>
                        <LeagueLevel>5</LeagueLevel>
                    </LeagueLevelUnit>
                    <NumberOfVictories>0</NumberOfVictories>
                    <NumberOfUndefeated>0</NumberOfUndefeated>
                    <Fanclub>
                        <FanclubID>0</FanclubID>
                        <FanclubName></FanclubName>
                        <FanclubSize>500</FanclubSize>
                    </Fanclub>
                    <LogoURL></LogoURL>
                    <!-- TeamColors missing here -->
                    <DressURI>//res.hattrick.org/kits/0/0/0/0/matchKitSmall.png</DressURI>
                    <DressAlternateURI>//res.hattrick.org/kits/0/0/0/1/matchKitSmall.png</DressAlternateURI>
                    <BotStatus>
                        <IsBot>False</IsBot>
                    </BotStatus>
                    <TeamRank>100</TeamRank>
                    <YouthTeamID>0</YouthTeamID>
                    <YouthTeamName></YouthTeamName>
                    <NumberOfVisits>1</NumberOfVisits>
                    <PossibleToChallengeMidweek>False</PossibleToChallengeMidweek>
                    <PossibleToChallengeWeekend>False</PossibleToChallengeWeekend>
                </Team>
            </Teams>
            <User>
               <UserID>2</UserID>
               <Language>
                 <LanguageID>1</LanguageID>
                 <LanguageName>English</LanguageName>
               </Language>
               <Name>Test User B</Name>
               <Loginname>TestUserB</Loginname>
               <SupporterTier>None</SupporterTier>
               <SignupDate>Date</SignupDate>
               <ActivationDate>Date</ActivationDate>
               <LastLoginDate>Date</LastLoginDate>
               <HasManagerLicense>True</HasManagerLicense>
            </User>
        </HattrickData>
        "#;

        let res: HattrickData =
            from_str(xml).expect("Failed to deserialize team XML without colors");
        let team = &res.Teams.Teams[0];

        assert_eq!(team.TeamID, "2000");
        assert!(team.TeamColors.is_none(), "TeamColors should be None");
    }
    #[test]
    fn test_deserialize_players_data() {
        let xml = r#"
        <HattrickData>
            <Team>
                <TeamID>3000</TeamID>
                <TeamName>Test Team C</TeamName>
                <ShortTeamName>Testers C</ShortTeamName>
                <PlayerList>
                    <Player>
                        <PlayerID>40001</PlayerID>
                        <FirstName>John</FirstName>
                        <LastName>Doe</LastName>
                        <PlayerNumber>1</PlayerNumber>
                        <Age>20</Age>
                        <AgeDays>10</AgeDays>
                        <TSI>1500</TSI>
                        <PlayerForm>6</PlayerForm>
                        <Statement></Statement>
                        <Experience>3</Experience>
                        <Loyalty>5</Loyalty>
                        <MotherClubBonus>True</MotherClubBonus>
                        <Leadership>5</Leadership>
                        <Salary>1000</Salary>
                        <IsAbroad>False</IsAbroad>
                        <Agreeability>3</Agreeability>
                        <Aggressiveness>3</Aggressiveness>
                        <Honesty>3</Honesty>
                        <LeagueGoals>0</LeagueGoals>
                        <CupGoals>0</CupGoals>
                        <FriendliesGoals>0</FriendliesGoals>
                        <CareerGoals>0</CareerGoals>
                        <CareerHattricks>0</CareerHattricks>
                        <Speciality>0</Speciality>
                        <TransferListed>False</TransferListed>
                        <NationalTeamID>0</NationalTeamID>
                        <CountryID>1</CountryID>
                        <Caps>0</Caps>
                        <CapsU20>0</CapsU20>
                        <Cards>0</Cards>
                        <InjuryLevel>-1</InjuryLevel>
                        <Sticker></Sticker>
                    </Player>
                    <Player>
                        <PlayerID>40002</PlayerID>
                        <FirstName>Jane</FirstName>
                        <LastName>Smith</LastName>
                        <PlayerNumber>10</PlayerNumber>
                        <Age>25</Age>
                        <TSI>5000</TSI>
                        <PlayerForm>7</PlayerForm>
                        <Experience>5</Experience>
                        <Loyalty>10</Loyalty>
                        <MotherClubBonus>False</MotherClubBonus>
                        <Leadership>3</Leadership>
                        <Salary>5000</Salary>
                        <IsAbroad>True</IsAbroad>
                        <Agreeability>3</Agreeability>
                        <Aggressiveness>3</Aggressiveness>
                        <Honesty>3</Honesty>
                        <TransferListed>True</TransferListed>
                        <CountryID>2</CountryID>
                    </Player>
                </PlayerList>
            </Team>
        </HattrickData>
        "#;

        let res: PlayersData = from_str(xml).expect("Failed to deserialize players data");
        assert_eq!(res.Team.TeamID, "3000");
        assert_eq!(res.Team.TeamName, "Test Team C");

        // Check optional short name
        assert_eq!(res.Team.ShortTeamName, Some("Testers C".to_string()));

        let player_list = res.Team.PlayerList.expect("PlayerList should exist");
        assert_eq!(player_list.players.len(), 2);

        let p1 = &player_list.players[0];
        assert_eq!(p1.PlayerID, 40001);
        assert_eq!(p1.FirstName, "John");
        assert_eq!(p1.MotherClubBonus, true);
        assert_eq!(p1.AgeDays, Some(10));

        let p2 = &player_list.players[1];
        assert_eq!(p2.PlayerID, 40002);
        assert_eq!(p2.FirstName, "Jane");
        assert_eq!(p2.IsAbroad, true);
        assert_eq!(p2.AgeDays, None); // Missing field test
        assert_eq!(p2.TransferListed, true);
    }

    #[test]
    fn test_teams_with_announcements_deserialization() {
        let xml = r#"<HattrickData>
      <FileName>teamdetails.xml</FileName>
      <Version>3.7</Version>
      <UserID>6992417</UserID>
      <FetchedDate>2026-02-01 18:12:26</FetchedDate>
      <User>
        <UserID>6992417</UserID>
        <Language>
          <LanguageID>2</LanguageID>
          <LanguageName>English (UK)</LanguageName>
        </Language>
        <SupporterTier>gold</SupporterTier>
        <Loginname>tychobrailleur</Loginname>
        <Name>HIDDEN</Name>
        <ICQ></ICQ>
        <SignupDate>2019-10-24 20:19:39</SignupDate>
        <ActivationDate>2019-10-24 20:20:00</ActivationDate>
        <LastLoginDate>2026-02-01 18:04:54</LastLoginDate>
        <HasManagerLicense>True</HasManagerLicense>
        <NationalTeams />
      </User>
      <Teams>
        <Team>
          <TeamID>1000</TeamID>
          <TeamName>Test Team A</TeamName>
          <ShortTeamName>A</ShortTeamName>
          <IsPrimaryClub>True</IsPrimaryClub>
          <FoundedDate>2019-10-24 20:20:00</FoundedDate>
          <IsDeactivated>False</IsDeactivated>
          <Arena>
            <ArenaID>1001</ArenaID>
            <ArenaName>Armadillo Arena</ArenaName>
          </Arena>
          <League>
            <LeagueID>21</LeagueID>
            <LeagueName>Ireland</LeagueName>
          </League>
          <Country>
            <CountryID>16</CountryID>
            <CountryName>Ireland</CountryName>
          </Country>
          <Region>
            <RegionID>627</RegionID>
            <RegionName>Wicklow</RegionName>
          </Region>
          <Trainer>
            <PlayerID>456835710</PlayerID>
          </Trainer>
          <HomePage></HomePage>
          <DressURI>//res.hattrick.org/kits/29/283/2827/2826328/matchKitSmall.png</DressURI>
          <DressAlternateURI>//res.hattrick.org/kits/29/283/2827/2826330/matchKitSmall.png</DressAlternateURI>
          <LeagueLevelUnit>
            <LeagueLevelUnitID>8806</LeagueLevelUnitID>
            <LeagueLevelUnitName>IV.32</LeagueLevelUnitName>
            <LeagueLevel>4</LeagueLevel>
          </LeagueLevelUnit>
          <BotStatus>
            <IsBot>False</IsBot>
          </BotStatus>
          <Cup />

          <PowerRating>
            <GlobalRanking>45701</GlobalRanking>
            <LeagueRanking>148</LeagueRanking>
            <RegionRanking>7</RegionRanking>
            <PowerRating>936</PowerRating>
          </PowerRating>
          <FriendlyTeamID />
          <NumberOfVictories />
          <NumberOfUndefeated />
          <TeamRank />
          <Fanclub>
            <FanclubID>673567</FanclubID>
            <FanclubName>Anteaters Ultras</FanclubName>
            <FanclubSize>2765</FanclubSize>
          </Fanclub>
          <LogoURL>//res.hattrick.org/teamlogo/3/29/281/280747/280747.png</LogoURL>
          <Guestbook>
            <NumberOfGuestbookItems>0</NumberOfGuestbookItems>
          </Guestbook>
          <PressAnnouncement>
            <Subject>The Bogdan Controversy</Subject>
            <Body>There was a certain amount of stupor within the Pangolins ranks when it was discovered in training camp today that there actually was a player whose first name was “Bogdan” on the team.

    “Come on lads” the player involved, [playerid=434668244]Tărtăreanu[/playerid], said “I have been here for 6 bloody seasons, how does it come as a surprise now??”

    The management team, still in shock, and busy sifting through piles of papers, refused to comment.</Body>
            <SendDate>2023-10-10 18:13:00</SendDate>
          </PressAnnouncement>
          <TeamColors>
            <BackgroundColor>288032</BackgroundColor>
            <Color>ffffff</Color>
          </TeamColors>
          <YouthTeamID>0</YouthTeamID>
          <YouthTeamName></YouthTeamName>
          <NumberOfVisits>0</NumberOfVisits>
          <TrophyList>
            <Trophy>
              <TrophyTypeId>17</TrophyTypeId>
              <TrophySeason>76</TrophySeason>
              <LeagueLevel>4</LeagueLevel>
              <LeagueLevelUnitId>8806</LeagueLevelUnitId>
              <LeagueLevelUnitName>IV.32</LeagueLevelUnitName>
              <GainedDate>2024-09-07 01:51:00</GainedDate>
              <ImageUrl>/App_Themes/Standard/Images/Trophies/iv.png</ImageUrl>
              <CupLeagueLevel></CupLeagueLevel>
              <CupLevel></CupLevel>
              <CupLevelIndex></CupLevelIndex>
            </Trophy>
            <Trophy>
              <TrophyTypeId>17</TrophyTypeId>
              <TrophySeason>75</TrophySeason>
              <LeagueLevel>4</LeagueLevel>
              <LeagueLevelUnitId>8806</LeagueLevelUnitId>
              <LeagueLevelUnitName>IV.32</LeagueLevelUnitName>
              <GainedDate>2024-05-18 01:52:00</GainedDate>
              <ImageUrl>/App_Themes/Standard/Images/Trophies/iv.png</ImageUrl>
              <CupLeagueLevel></CupLeagueLevel>
              <CupLevel></CupLevel>
              <CupLevelIndex></CupLevelIndex>
            </Trophy>
            <Trophy>
              <TrophyTypeId>17</TrophyTypeId>
              <TrophySeason>74</TrophySeason>
              <LeagueLevel>4</LeagueLevel>
              <LeagueLevelUnitId>8806</LeagueLevelUnitId>
              <LeagueLevelUnitName>IV.32</LeagueLevelUnitName>
              <GainedDate>2024-01-27 01:51:00</GainedDate>
              <ImageUrl>/App_Themes/Standard/Images/Trophies/iv.png</ImageUrl>
              <CupLeagueLevel></CupLeagueLevel>
              <CupLevel></CupLevel>
              <CupLevelIndex></CupLevelIndex>
            </Trophy>
            <Trophy>
              <TrophyTypeId>17</TrophyTypeId>
              <TrophySeason>73</TrophySeason>
              <LeagueLevel>4</LeagueLevel>
              <LeagueLevelUnitId>8806</LeagueLevelUnitId>
              <LeagueLevelUnitName>IV.32</LeagueLevelUnitName>
              <GainedDate>2023-10-07 01:51:00</GainedDate>
              <ImageUrl>/App_Themes/Standard/Images/Trophies/iv.png</ImageUrl>
              <CupLeagueLevel></CupLeagueLevel>
              <CupLevel></CupLevel>
              <CupLevelIndex></CupLevelIndex>
            </Trophy>
            <Trophy>
              <TrophyTypeId>17</TrophyTypeId>
              <TrophySeason>70</TrophySeason>
              <LeagueLevel>4</LeagueLevel>
              <LeagueLevelUnitId>8826</LeagueLevelUnitId>
              <LeagueLevelUnitName>IV.52</LeagueLevelUnitName>
              <GainedDate>2022-11-05 01:52:00</GainedDate>
              <ImageUrl>/App_Themes/Standard/Images/Trophies/iv.png</ImageUrl>
              <CupLeagueLevel></CupLeagueLevel>
              <CupLevel></CupLevel>
              <CupLevelIndex></CupLevelIndex>
            </Trophy>
            <Trophy>
              <TrophyTypeId>17</TrophyTypeId>
              <TrophySeason>68</TrophySeason>
              <LeagueLevel>4</LeagueLevel>
              <LeagueLevelUnitId>8810</LeagueLevelUnitId>
              <LeagueLevelUnitName>IV.36</LeagueLevelUnitName>
              <GainedDate>2022-03-26 01:51:00</GainedDate>
              <ImageUrl>/App_Themes/Standard/Images/Trophies/iv.png</ImageUrl>
              <CupLeagueLevel></CupLeagueLevel>
              <CupLevel></CupLevel>
              <CupLevelIndex></CupLevelIndex>
            </Trophy>
            <Trophy>
              <TrophyTypeId>17</TrophyTypeId>
              <TrophySeason>67</TrophySeason>
              <LeagueLevel>4</LeagueLevel>
              <LeagueLevelUnitId>8810</LeagueLevelUnitId>
              <LeagueLevelUnitName>IV.36</LeagueLevelUnitName>
              <GainedDate>2021-12-04 01:51:00</GainedDate>
              <ImageUrl>/App_Themes/Standard/Images/Trophies/iv.png</ImageUrl>
              <CupLeagueLevel></CupLeagueLevel>
              <CupLevel></CupLevel>
              <CupLevelIndex></CupLevelIndex>
            </Trophy>
            <Trophy>
              <TrophyTypeId>17</TrophyTypeId>
              <TrophySeason>65</TrophySeason>
              <LeagueLevel>4</LeagueLevel>
              <LeagueLevelUnitId>8789</LeagueLevelUnitId>
              <LeagueLevelUnitName>IV.15</LeagueLevelUnitName>
              <GainedDate>2021-04-24 02:18:00</GainedDate>
              <ImageUrl>/App_Themes/Standard/Images/Trophies/iv.png</ImageUrl>
              <CupLeagueLevel></CupLeagueLevel>
              <CupLevel></CupLevel>
              <CupLevelIndex></CupLevelIndex>
            </Trophy>
            <Trophy>
              <TrophyTypeId>17</TrophyTypeId>
              <TrophySeason>61</TrophySeason>
              <LeagueLevel>5</LeagueLevel>
              <LeagueLevelUnitId>34587</LeagueLevelUnitId>
              <LeagueLevelUnitName>V.5</LeagueLevelUnitName>
              <GainedDate>2020-02-01 02:44:00</GainedDate>
              <ImageUrl>/App_Themes/Standard/Images/Trophies/v.png</ImageUrl>
              <CupLeagueLevel></CupLeagueLevel>
              <CupLevel></CupLevel>
              <CupLevelIndex></CupLevelIndex>
            </Trophy>
          </TrophyList>
          <PossibleToChallengeMidweek>False</PossibleToChallengeMidweek>
          <PossibleToChallengeWeekend>False</PossibleToChallengeWeekend>
        </Team>
      </Teams>
    </HattrickData>"#;

        let res: HattrickData = from_str(xml).expect("Failed to deserialize HattrickData data");
        let team_data = &res.Teams.Teams[0];

        assert_eq!(team_data.TeamID, "1000");
        assert_eq!(team_data.TeamName, "Test Team A");
        assert_eq!(team_data.IsPrimaryClub, Some(true));
        assert_eq!(team_data.IsDeactivated, Some(false));
    }

    // Leaving this as sanity check, this what I used to debug the empty tag issue...
    #[derive(Debug, Deserialize)]
    struct Example {
        #[serde(default, deserialize_with = "deserialize_empty_tag_is_none")]
        pub EmptyTag: Option<u32>,
    }

    #[test]
    fn test_deserialize_team_with_empty_tags() {
        let xml = r#"<Example>
        <EmptyTag />
        </Example>"#;

        let res: Example = from_str(xml).expect("Failed to deserialize Example data");
        assert_eq!(res.EmptyTag, None);

        let xml2 = r#"<Example>
        </Example>"#;

        let res2: Example = from_str(xml2).expect("Failed to deserialize Example data");
        assert_eq!(res2.EmptyTag, None);
    }

    #[test]
    fn test_merge_player_data_with_detailed() {
        use crate::chpp::model::Player;

        // Create basic player with some fields
        let basic = Player {
            PlayerID: 1,
            FirstName: "John".to_string(),
            LastName: "Doe".to_string(),
            NickName: None,
            PlayerNumber: Some(10),
            Age: 25,
            AgeDays: Some(100),
            TSI: 1000,
            PlayerForm: 5,
            Statement: Some("Basic statement".to_string()),
            Experience: 3,
            Loyalty: 10,
            ReferencePlayerID: Some(999),
            MotherClubBonus: false,
            Leadership: 3,
            Salary: 500,
            IsAbroad: false,
            Agreeability: 3,
            Aggressiveness: 3,
            Honesty: 3,
            LeagueGoals: Some(5),
            CupGoals: Some(2),
            FriendliesGoals: Some(1),
            CareerGoals: Some(50),
            CareerHattricks: Some(2),
            Speciality: Some(1),
            TransferListed: false,
            NationalTeamID: Some(100),
            CountryID: Some(10),
            Caps: Some(5),
            CapsU20: Some(10),
            Cards: Some(1),
            InjuryLevel: Some(-1),
            Sticker: Some("Basic sticker".to_string()),
            Flag: None,
            PlayerSkills: None,
            LastMatch: None,
            ArrivalDate: None,
            PlayerCategoryId: None,
            MotherClub: None,
            NativeCountryID: None,
            NativeLeagueID: None,
            NativeLeagueName: None,
            MatchesCurrentTeam: None,
            GoalsCurrentTeam: None,
            AssistsCurrentTeam: None,
            CareerAssists: None,
        };

        // Create detailed player with most fields but some missing
        let detailed = Player {
            PlayerID: 1,
            FirstName: "John".to_string(),
            LastName: "Doe".to_string(),
            NickName: Some("H.".to_string()),
            PlayerNumber: None, // Missing in detailed
            Age: 25,
            AgeDays: None, // Missing in detailed
            TSI: 1500,     // Different value
            PlayerForm: 6, // Different value
            Statement: Some("Detailed statement".to_string()),
            Experience: 4,
            Loyalty: 11,
            ReferencePlayerID: None, // Missing in detailed
            MotherClubBonus: false,
            Leadership: 4,
            Salary: 600,
            IsAbroad: false,
            Agreeability: 4,
            Aggressiveness: 4,
            Honesty: 4,
            LeagueGoals: Some(6),
            CupGoals: None, // Missing in detailed
            FriendliesGoals: Some(2),
            CareerGoals: Some(55),
            CareerHattricks: None, // Missing in detailed
            Speciality: Some(1),
            TransferListed: false,
            NationalTeamID: Some(100),
            CountryID: Some(10),
            Caps: Some(6),
            CapsU20: None, // Missing in detailed
            Cards: Some(1),
            InjuryLevel: Some(0),
            Sticker: None, // Missing in detailed
            Flag: None,
            PlayerSkills: Some(crate::chpp::model::PlayerSkills {
                StaminaSkill: 7,
                KeeperSkill: 1,
                PlaymakerSkill: 5,
                ScorerSkill: 6,
                PassingSkill: 5,
                WingerSkill: 4,
                DefenderSkill: 3,
                SetPiecesSkill: 4,
            }),
            LastMatch: None,
            ArrivalDate: None,
            PlayerCategoryId: None,
            MotherClub: None,
            NativeCountryID: None,
            NativeLeagueID: None,
            NativeLeagueName: None,
            MatchesCurrentTeam: None,
            GoalsCurrentTeam: None,
            AssistsCurrentTeam: None,
            CareerAssists: None,
        };

        let merged = basic.merge_player_data(Some(detailed));

        // Verify detailed data is primary
        assert_eq!(merged.TSI, 1500);
        assert_eq!(merged.PlayerForm, 6);
        assert_eq!(merged.Statement, Some("Detailed statement".to_string()));
        assert!(merged.PlayerSkills.is_some());

        // Verify missing fields filled from basic
        assert_eq!(merged.PlayerNumber, Some(10)); // From basic
        assert_eq!(merged.AgeDays, Some(100)); // From basic
        assert_eq!(merged.ReferencePlayerID, Some(999)); // From basic
        assert_eq!(merged.CupGoals, Some(2)); // From basic
        assert_eq!(merged.CareerHattricks, Some(2)); // From basic
        assert_eq!(merged.CapsU20, Some(10)); // From basic
        assert_eq!(merged.Sticker, Some("Basic sticker".to_string())); // From basic
    }

    #[test]
    fn test_merge_player_data_without_detailed() {
        use crate::chpp::model::Player;

        let basic = Player {
            PlayerID: 1,
            FirstName: "John".to_string(),
            LastName: "Doe".to_string(),
            NickName: None,
            PlayerNumber: Some(10),
            Age: 25,
            AgeDays: Some(100),
            TSI: 1000,
            PlayerForm: 5,
            Statement: Some("Basic statement".to_string()),
            Experience: 3,
            Loyalty: 10,
            ReferencePlayerID: Some(999),
            MotherClubBonus: false,
            Leadership: 3,
            Salary: 500,
            IsAbroad: false,
            Agreeability: 3,
            Aggressiveness: 3,
            Honesty: 3,
            LeagueGoals: Some(5),
            CupGoals: Some(2),
            FriendliesGoals: Some(1),
            CareerGoals: Some(50),
            CareerHattricks: Some(2),
            Speciality: Some(1),
            TransferListed: false,
            NationalTeamID: Some(100),
            CountryID: Some(10),
            Caps: Some(5),
            CapsU20: Some(10),
            Cards: Some(1),
            InjuryLevel: Some(-1),
            Sticker: Some("Basic sticker".to_string()),
            Flag: None,
            PlayerSkills: None,
            LastMatch: None,
            ArrivalDate: None,
            PlayerCategoryId: None,
            MotherClub: None,
            NativeCountryID: None,
            NativeLeagueID: None,
            NativeLeagueName: None,
            MatchesCurrentTeam: None,
            GoalsCurrentTeam: None,
            AssistsCurrentTeam: None,
            CareerAssists: None,
        };

        let merged = basic.merge_player_data(None);

        // Should be identical to basic
        assert_eq!(merged.PlayerID, basic.PlayerID);
        assert_eq!(merged.TSI, basic.TSI);
        assert_eq!(merged.PlayerNumber, basic.PlayerNumber);
        assert_eq!(merged.Statement, basic.Statement);
        assert!(merged.PlayerSkills.is_none());
    }
}
