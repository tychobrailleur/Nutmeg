use serde::Deserialize;
//use uuid::Uuid;

// Team Details:
// https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=teamdetails

// Utility function for deserialisation
fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error> where D: serde::Deserializer<'de> {
    let s: String = Deserialize::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(serde::de::Error::custom(format!("Expected True or False, got {}", s))),
    }
}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq)]
pub enum SupporterTier {
    None,
    Silver,
    Gold,
    Platinum,
    Diamond
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
            _ => Err(serde::de::Error::custom(format!("Unknown SupporterTier: {}", s))),
        }
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Language {
    pub LanguageID: u32,
    pub LanguageName: String
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct User {
    #[allow(dead_code)]
    pub UserID:u32,
    #[allow(dead_code)]
    pub Language: Language,
    pub Name:String,
    pub Loginname:String,
    pub SupporterTier:SupporterTier,
    pub SignupDate:String,
    pub ActivationDate: String,
    pub LastLoginDate: String,
    #[serde(deserialize_with = "deserialize_bool")]
    pub HasManagerLicense: bool
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Arena {
    pub ArenaID: u32,
    pub ArenaName: String
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct League {
    pub LeagueID: u32,
    pub LeagueName: String
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Country {
    pub CountryID: u32,
    pub CountryName: String
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Region {
    pub RegionID: u32,
    pub RegionName: String
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Trainer {
    pub PlayerID: u32
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Fanclub {
    pub FanclubID: u32,
    pub FanclubName: String,
    pub FanclubSize: u32
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Cup {
    #[serde(deserialize_with = "deserialize_bool")]
    pub StillInCup: bool,
    pub CupID: Option<u32>,
    pub CupName: Option<String>,
    pub CupLeagueLevel: Option<u32>, // 0 = National (LeagueLevel 1-6), 7-9 = Divisional.
    pub CupLevel: Option<u32>,       // 1 = National/Divisional, 2 = Challenger, 3 = Consolation.
    pub CupLevelIndex: Option<u32>,  // Always 1 for National and Consolation cups, for Challenger cup: 1 = Emerald, 2 = Ruby, 3 = Sapphire
    pub MatchRound: Option<u32>,
    pub MatchRoundsLeft: Option<u32>
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct LeagueLevelUnit {
    pub LeagueLevelUnitID: u32,
    pub LeagueLevelUnitName: String,
    pub LeagueLevel: u32
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct PowerRating {
    pub GlobalRanking: u32,
    pub LeagueRanking: u32,
    pub RegionRanking: u32,
    pub PowerRating: u32
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct TeamColors {
    pub BackgroundColor: String,
    pub Color: String
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct BotStatus {
    #[serde(deserialize_with = "deserialize_bool")]
    pub IsBot: bool,
    pub BotSince: Option<String>
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
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
#[derive(Deserialize, Debug)]
pub struct TrophyListWrapper {
    #[serde(rename = "Trophy", default)]
    pub trophies: Vec<Trophy>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Team {
    pub TeamID: String,
    pub TeamName: String,
    pub ShortTeamName: String,
    #[serde(deserialize_with = "deserialize_bool")]
    pub IsPrimaryClub: bool,
    pub FoundedDate: String,
    #[serde(deserialize_with = "deserialize_bool")]
    pub IsDeactivated: bool,
    pub Arena: Arena,
    pub League: League,
    pub Country: Country,
    pub Region: Region,
    pub Trainer: Trainer,
    pub HomePage: String,
    pub Cup: Option<Cup>,
    pub PowerRating: PowerRating,
    pub FriendlyTeamID: u32,
    pub LeagueLevelUnit: LeagueLevelUnit,
    pub NumberOfVictories: u32,
    pub NumberOfUndefeated: u32,
    pub Fanclub: Fanclub,
    pub LogoURL: String,
    pub TeamColors: TeamColors,
    pub DressURI: String,
    pub DressAlternateURI: String,
    pub BotStatus: BotStatus,
    pub TeamRank: u32,
    pub YouthTeamID: u32,
    pub YouthTeamName: String,
    pub NumberOfVisits: u32,
    //  pub Flags: Flags,
    //#[serde(rename = "TrophyList")]
    //pub TrophyList: Option<TrophyListWrapper>,
    #[serde(deserialize_with = "deserialize_bool")]
    pub PossibleToChallengeMidweek: bool,
    #[serde(deserialize_with = "deserialize_bool")]
    pub PossibleToChallengeWeekend: bool
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Teams {
    #[serde(rename = "Team")]
    pub Teams:Vec<Team>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct HattrickData {
    pub Teams:Teams,
    #[allow(dead_code)]
    pub User:User
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_xml_rs::from_str;

    #[test]
    fn test_deserialize_bool_true() {
        #[derive(Deserialize, Debug, PartialEq)]
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
        #[derive(Deserialize, Debug, PartialEq)]
        struct BooleanWrapper {
            #[serde(deserialize_with = "deserialize_bool")]
            val: bool,
        }

        let xml = "<BooleanWrapper><val>False</val></BooleanWrapper>";
        let res: BooleanWrapper = from_str(xml).unwrap();
        assert_eq!(res.val, false);
    }

    #[test]
    fn test_supporter_tier_case_insensitive() {
        #[derive(Deserialize, Debug, PartialEq)]
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
}
