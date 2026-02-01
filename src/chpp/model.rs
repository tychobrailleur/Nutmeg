/* model.rs
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

use serde::{Deserialize, Serialize};
//use uuid::Uuid;

// Team Details:
// https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=teamdetails

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
#[derive(Serialize, Debug, PartialEq)]
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
#[derive(Deserialize, Serialize, Debug)]
pub struct Language {
    pub LanguageID: u32,
    pub LanguageName: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
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
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct League {
    pub LeagueID: u32,
    pub LeagueName: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Country {
    pub CountryID: u32,
    pub CountryName: String,
    pub Currency: Option<Currency>,
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
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub StillInCup: bool,
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

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LastMatch {
    pub Date: String,
    pub MatchId: u32,
    pub PositionCode: u32,
    pub PlayedMinutes: u32,
    pub Rating: Option<u32>,
    pub RatingEndOfMatch: Option<u32>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Player {
    pub PlayerID: u32,
    pub FirstName: String,
    pub LastName: String,
    pub PlayerNumber: u32,
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
    pub Speciality: Option<u32>,
    #[serde(
        deserialize_with = "deserialize_bool",
        serialize_with = "serialize_bool"
    )]
    pub TransferListed: bool,
    pub NationalTeamID: Option<u32>,
    pub CountryID: u32,
    pub Caps: Option<u32>,
    pub CapsU20: Option<u32>,
    pub Cards: Option<u32>,
    pub InjuryLevel: Option<i32>, // -1 = No injury, 0 = Bruised, >0 = Weeks
    pub Sticker: Option<String>,
    pub PlayerSkills: Option<PlayerSkills>, // Only visible for own team or if authorized
    pub LastMatch: Option<LastMatch>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlayerList {
    #[serde(rename = "Player")]
    pub players: Vec<Player>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
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
    pub FriendlyTeamID: Option<u32>,
    pub LeagueLevelUnit: Option<LeagueLevelUnit>,
    pub NumberOfVictories: Option<u32>,
    pub NumberOfUndefeated: Option<u32>,
    pub Fanclub: Option<Fanclub>,
    pub LogoURL: Option<String>,
    pub TeamColors: Option<TeamColors>,
    pub DressURI: Option<String>,
    pub DressAlternateURI: Option<String>,
    pub BotStatus: Option<BotStatus>,
    pub TeamRank: Option<u32>,
    pub YouthTeamID: Option<u32>,
    pub YouthTeamName: Option<String>,
    pub NumberOfVisits: Option<u32>,
    //  pub Flags: Flags,
    //#[serde(rename = "TrophyList")]
    //pub TrophyList: Option<TrophyListWrapper>,
    pub PlayerList: Option<PlayerList>,
    #[serde(deserialize_with = "deserialize_option_bool", default)]
    pub PossibleToChallengeMidweek: Option<bool>,
    #[serde(deserialize_with = "deserialize_option_bool", default)]
    pub PossibleToChallengeWeekend: Option<bool>,
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
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct WorldLeague {
    pub LeagueID: u32,
    pub LeagueName: String,
    pub Country: WorldCountry,
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
}
