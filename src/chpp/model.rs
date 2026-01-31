use serde::Deserialize;
//use uuid::Uuid;


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
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SupporterTier {
    None,
    Silver,
    Gold,
    Platinum,
    Diamond
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct User {
    #[allow(dead_code)]
    pub UserID:u32,
    #[allow(dead_code)]
    pub Name:String,
    pub Loginname:String,
    pub SupporterTier:SupporterTier,
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
    pub League: League
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
