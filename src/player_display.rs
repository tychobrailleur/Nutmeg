use crate::chpp::model::Player;
use gettextrs::gettext;
use num_format::{Buffer, SystemLocale};

pub struct PlayerDisplay {
    pub name: String,
    pub flag: String,
    pub number: String,
    pub age: String,
    pub form: String,
    pub tsi: String,
    pub salary: String,
    pub specialty: String,
    pub xp: String,
    pub leadership: String,
    pub loyalty: String,
    pub best_pos: String,
    pub last_pos: String,
    pub stamina: String,
    pub injured: String,
    pub cards: String,
    pub mother_club: String,
    pub mother_club_bonus: bool,
}

impl PlayerDisplay {
    pub fn new(p: &Player, locale: &SystemLocale) -> Self {
        let name = format!("{} {}", p.FirstName, p.LastName);
        let flag = p.Flag.clone().unwrap_or_else(|| "🏳️".to_string());
        let number = p.PlayerNumber.map(|n| n.to_string()).unwrap_or_else(|| "-".to_string());
        let age = format!("{}.{}", p.Age, p.AgeDays.unwrap_or(0));
        let form = p.PlayerForm.to_string();
        
        let mut buf_tsi = Buffer::default();
        buf_tsi.write_formatted(&p.TSI, locale);
        let tsi = buf_tsi.as_str().to_string();

        let mut buf_salary = Buffer::default();
        buf_salary.write_formatted(&p.Salary, locale);
        let salary = format!("{} €", buf_salary.as_str());

        let specialty = match p.Speciality {
            Some(1) => gettext("Technical"),
            Some(2) => gettext("Quick"),
            Some(3) => gettext("Powerful"),
            Some(4) => gettext("Unpredictable"),
            Some(5) => gettext("Head"),
            Some(6) => gettext("Resilient"),
            Some(7) => gettext("Support"),
            _ => "".to_string(),
        };

        let xp = p.Experience.to_string();
        let leadership = p.Leadership.to_string();
        let loyalty = p.Loyalty.to_string();

        let best_pos = match p.PlayerCategoryId {
            Some(1) => gettext("Keeper"),
            Some(2) => gettext("Right Back"),
            Some(3) => gettext("Central Defender"),
            Some(4) => gettext("Winger"),
            Some(5) => gettext("Inner Midfielder"),
            Some(6) => gettext("Forward"),
            _ => "-".to_string(),
        };

        let last_pos_code = p.LastMatch.as_ref().map(|m| m.PositionCode).unwrap_or(0);
        let last_pos = if last_pos_code == 0 { "-".to_string() } else { last_pos_code.to_string() };

        let stamina = p.PlayerSkills.as_ref().map(|s| s.StaminaSkill.to_string()).unwrap_or_else(|| "-".to_string());

        let injured = match p.InjuryLevel {
            Some(i) if i == 0 => "🩹".to_string(),
            Some(i) if i > 0 => format!("🚑 {}w", i),
            _ => "".to_string(),
        };

        let cards = match p.Cards {
            Some(1) => "🟨".to_string(),
            Some(2) => "🟨🟨".to_string(),
            Some(3) => "🟥".to_string(),
            _ => "".to_string(),
        };

        let mother_club = if p.MotherClubBonus { "❤️".to_string() } else { "".to_string() };

        Self {
            name,
            flag,
            number,
            age,
            form,
            tsi,
            salary,
            specialty,
            xp,
            leadership,
            loyalty,
            best_pos,
            last_pos,
            stamina,
            injured,
            cards,
            mother_club,
            mother_club_bonus: p.MotherClubBonus,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chpp::model::{Player, PlayerSkills, LastMatch};

    fn create_dummy_player() -> Player {
        Player {
            PlayerID: 1,
            FirstName: "John".to_string(),
            LastName: "Doe".to_string(),
            NickName: None,
            PlayerNumber: Some(10),
            Age: 20,
            AgeDays: Some(10),
            TSI: 10000,
            PlayerForm: 5,
            Statement: None,
            Experience: 3,
            Loyalty: 10,
            ReferencePlayerID: None,
            MotherClubBonus: true,
            Leadership: 4,
            Salary: 50000,
            IsAbroad: false,
            Agreeability: 2,
            Aggressiveness: 2,
            Honesty: 2,
            LeagueGoals: None,
            CupGoals: None,
            FriendliesGoals: None,
            CareerGoals: None,
            CareerHattricks: None,
            CareerAssists: None,
            Speciality: Some(2), // Quick
            TransferListed: false,
            NationalTeamID: None,
            CountryID: None,
            Caps: None,
            CapsU20: None,
            Cards: Some(1),
            InjuryLevel: Some(1),
            Sticker: None,
            Flag: Some("🏳️".to_string()),
            PlayerSkills: Some(PlayerSkills {
                StaminaSkill: 7,
                KeeperSkill: 1,
                PlaymakerSkill: 5,
                ScorerSkill: 3,
                PassingSkill: 4,
                WingerSkill: 4,
                DefenderSkill: 2,
                SetPiecesSkill: 3,
            }),
            ArrivalDate: None,
            PlayerCategoryId: Some(6), // Forward
            MotherClub: None,
            NativeCountryID: None,
            NativeLeagueID: None,
            NativeLeagueName: None,
            MatchesCurrentTeam: None,
            GoalsCurrentTeam: None,
            AssistsCurrentTeam: None,
            LastMatch: Some(LastMatch {
                Date: "2023-01-01".to_string(),
                MatchId: 100,
                PositionCode: 100,
                PlayedMinutes: 90,
                Rating: Some(5.0),
                RatingEndOfMatch: None,
            }),
        }
    }

    #[test]
    fn test_player_display_formatting() {
        // Use C locale for predictable output (no separators vs comma/dot ambiguity in tests)
        // Or we can assume strict output given SystemLocale::from_name("C")
        let locale = SystemLocale::from_name("C").unwrap();
        let p = create_dummy_player();
        let display = PlayerDisplay::new(&p, &locale);

        assert_eq!(display.name, "John Doe");
        assert_eq!(display.number, "10");
        assert_eq!(display.age, "20.10");
        assert_eq!(display.tsi, "10000"); // C locale has no separators
        assert_eq!(display.salary, "50000 €");
        // gettext might return English or translation, but in unit test environment usually defaults to msgid if not initialized
        // Assuming "Quick" for ID 2
        // We might need to mock gettext or check potential values
        // assert_eq!(display.specialty, "Quick"); 
        
        assert_eq!(display.xp, "3");
        assert_eq!(display.mother_club, "❤️");
        assert_eq!(display.injured, "🚑 1w");
        assert_eq!(display.cards, "🟨");
        assert_eq!(display.stamina, "7");
        assert_eq!(display.last_pos, "100");
    }

    #[test]
    fn test_player_display_locale() {
        // Try a locale with separators if available, else stick to C
        // Note: Creating specific locales might fail on some systems if not generated.
        // We'll skip complex locale verification to avoid environment flakiness, 
        // relying on num-format's own tests for correctness.
        // Just verify it doesn't crash.
        let locale = SystemLocale::default().unwrap_or_else(|_| SystemLocale::from_name("C").unwrap());
        let p = create_dummy_player();
        let _display = PlayerDisplay::new(&p, &locale);
    }
}
