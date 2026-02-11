use super::types::PlayerSkill;
/// Player strength calculations
/// Implements the core formulas for converting player skills to rating strength
use crate::chpp::model::Player;

/// Calculate skill rating from displayed skill value
/// Excellent without sub skill gives 7.0
/// Formula: max(0, skill - 1)
pub fn calc_skill_rating(skill: f64) -> f64 {
    (skill - 1.0).max(0.0)
}

/// Calculate player's loyalty impact on rating
/// Home grown players get 1.5 bonus
/// Other players: loyalty_rating / 19
pub fn calc_loyalty(player: &Player) -> f64 {
    if player.MotherClubBonus {
        1.5
    } else {
        let loyalty_skill = player.Loyalty as f64;
        calc_skill_rating(loyalty_skill) / 19.0
    }
}

/// Calculate player's form impact on rating
/// Form range: 0.5 .. 7.0
/// Formula: 0.378 * sqrt(min(7, skill_rating))
pub fn calc_form(player: &Player) -> f64 {
    let form_skill = player.PlayerForm as f64;
    let form_rating = calc_skill_rating(form_skill).min(7.0);
    0.378 * form_rating.sqrt()
}

/// Calculate player's overall strength for a given skill
/// Formula: (skill_rating + loyalty) * form
pub fn calc_strength(player: &Player, skill: PlayerSkill) -> f64 {
    let skill_value = get_player_skill(player, skill);
    let skill_rating = calc_skill_rating(skill_value);
    let loyalty = calc_loyalty(player);
    let form = calc_form(player);

    (skill_rating + loyalty) * form
}

/// Get player's skill value for a given skill type
fn get_player_skill(player: &Player, skill: PlayerSkill) -> f64 {
    let skills = player.PlayerSkills.as_ref();
    match skill {
        PlayerSkill::Keeper => skills.map(|s| s.KeeperSkill).unwrap_or(0) as f64,
        PlayerSkill::Defending => skills.map(|s| s.DefenderSkill).unwrap_or(0) as f64,
        PlayerSkill::Playmaking => skills.map(|s| s.PlaymakerSkill).unwrap_or(0) as f64,
        PlayerSkill::Passing => skills.map(|s| s.PassingSkill).unwrap_or(0) as f64,
        PlayerSkill::Winger => skills.map(|s| s.WingerSkill).unwrap_or(0) as f64,
        PlayerSkill::Scoring => skills.map(|s| s.ScorerSkill).unwrap_or(0) as f64,
        PlayerSkill::SetPieces => skills.map(|s| s.SetPiecesSkill).unwrap_or(0) as f64,
        PlayerSkill::Form => player.PlayerForm as f64,
        PlayerSkill::Stamina => skills.map(|s| s.StaminaSkill).unwrap_or(0) as f64,
        PlayerSkill::Experience => player.Experience as f64,
        PlayerSkill::Loyalty => player.Loyalty as f64,
    }
}

/// Calculate player's tactic strength (includes experience)
/// Used for set pieces and other tactical calculations
pub fn calc_player_tactic_strength(player: &Player, skill: PlayerSkill) -> f64 {
    let loyalty = calc_loyalty(player);
    let strength = calc_strength(player, skill);
    let exp_skill = player.Experience as f64;
    let xp = calc_skill_rating(exp_skill);
    let f = xp.log10() * 4.0 / 3.0;

    loyalty + strength + f
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chpp::model::PlayerSkills;

    /// Helper to create a test player for strength calculations
    fn create_test_player(
        form: u32,
        experience: u32,
        loyalty: u32,
        mother_club_bonus: bool,
        keeper: u32,
        defender: u32,
    ) -> Player {
        Player {
            PlayerID: 1,
            FirstName: "Test".to_string(),
            LastName: "Player".to_string(),
            NickName: None,
            PlayerNumber: Some(1),
            Age: 25,
            AgeDays: Some(0),
            TSI: 1000,
            PlayerForm: form,
            Statement: None,
            Experience: experience,
            Loyalty: loyalty,
            ReferencePlayerID: None,
            MotherClubBonus: mother_club_bonus,
            Leadership: 5,
            Salary: 1000,
            IsAbroad: false,
            Agreeability: 3,
            Aggressiveness: 3,
            Honesty: 3,
            LeagueGoals: Some(0),
            CupGoals: Some(0),
            FriendliesGoals: Some(0),
            CareerGoals: Some(0),
            CareerHattricks: Some(0),
            CareerAssists: Some(0),
            Speciality: None,
            TransferListed: false,
            NationalTeamID: None,
            CountryID: Some(1),
            Caps: None,
            CapsU20: None,
            Cards: Some(0),
            InjuryLevel: Some(-1),
            Sticker: None,
            Flag: None,
            PlayerSkills: Some(PlayerSkills {
                KeeperSkill: keeper,
                DefenderSkill: defender,
                PlaymakerSkill: 7,
                ScorerSkill: 6,
                PassingSkill: 7,
                WingerSkill: 5,
                SetPiecesSkill: 6,
                StaminaSkill: 7,
            }),
            ArrivalDate: None,
            PlayerCategoryId: Some(2),
            MotherClub: None,
            NativeCountryID: Some(1),
            NativeLeagueID: Some(1),
            NativeLeagueName: Some("Test League".to_string()),
            MatchesCurrentTeam: Some(0),
            GoalsCurrentTeam: Some(0),
            AssistsCurrentTeam: Some(0),
            LastMatch: None,
        }
    }

    #[test]
    fn test_calc_skill_rating() {
        assert_eq!(calc_skill_rating(1.0), 0.0); // Disastrous
        assert_eq!(calc_skill_rating(8.0), 7.0); // Excellent (no sub)
        assert_eq!(calc_skill_rating(9.0), 8.0); // Excellent + 1 sub
        assert_eq!(calc_skill_rating(0.5), 0.0); // Below 1
        assert_eq!(calc_skill_rating(15.0), 14.0); // Titanic
    }

    #[test]
    fn test_calc_loyalty_mother_club() {
        let player = create_test_player(7, 5, 10, true, 8, 10);
        
        // Mother club bonus gives fixed 1.5
        assert_eq!(calc_loyalty(&player), 1.5);
    }

    #[test]
    fn test_calc_loyalty_non_mother_club() {
        // Loyalty = 8 (Excellent with no sub)
        let player = create_test_player(7, 5, 8, false, 8, 10);
        
        // calc_skill_rating(8) = 7.0
        // loyalty_rating = 7.0 / 19.0 ≈ 0.368
        let loyalty = calc_loyalty(&player);
        assert!((loyalty - 0.368).abs() < 0.001);
    }

    #[test]
    fn test_calc_loyalty_low() {
        // Loyalty = 1 (Disastrous)
        let player = create_test_player(7, 5, 1, false, 8, 10);
        
        // calc_skill_rating(1) = 0.0
        // loyalty_rating = 0.0 / 19.0 = 0.0
        assert_eq!(calc_loyalty(&player), 0.0);
    }

    #[test]
    fn test_calc_form_excellent() {
        // Form = 8 (Excellent)
        let player = create_test_player(8, 5, 10, false, 8, 10);
        
        // calc_skill_rating(8) = 7.0, min(7, 7.0) = 7.0
        // form = 0.378 * sqrt(7.0) ≈ 0.378 * 2.646 ≈ 1.0
        let form = calc_form(&player);
        assert!((form - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_calc_form_solid() {
        // Form = 7 (Solid)
        let player = create_test_player(7, 5, 10, false, 8, 10);
        
        // calc_skill_rating(7) = 6.0, min(7, 6.0) = 6.0
        // form = 0.378 * sqrt(6.0) ≈ 0.926
        let form = calc_form(&player);
        assert!((form - 0.926).abs() < 0.01);
    }

    #[test]
    fn test_calc_form_caps_at_seven() {
        // Form = 10 (Outstanding) - but should be capped at 7
        let player = create_test_player(10, 5, 10, false, 8, 10);
        
        // calc_skill_rating(10) = 9.0, min(7, 9.0) = 7.0
        // form = 0.378 * sqrt(7.0) ≈ 1.0
        let form = calc_form(&player);
        assert!((form - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_calc_strength_basic() {
        // Form=8, Loyalty=10, Keeper=10
        let player = create_test_player(8, 5, 10, false, 10, 12);
        
        let strength = calc_strength(&player, PlayerSkill::Keeper);
        
        // keeper_rating = calc_skill_rating(10) = 9.0
        // loyalty = calc_skill_rating(10) / 19.0 ≈ 0.474
        // form = 0.378 * sqrt(7) ≈ 1.0
        // strength = (9.0 + 0.474) * 1.0 ≈ 9.474
        assert!((strength - 9.474).abs() < 0.01);
    }

    #[test]
    fn test_calc_strength_mother_club() {
        // Form=8, Loyalty=10 (mother club), Keeper=10
        let player = create_test_player(8, 5, 10, true, 10, 12);
        
        let strength = calc_strength(&player, PlayerSkill::Keeper);
        
        // keeper_rating = 9.0
        // loyalty = 1.5 (mother club bonus)
        // form ≈ 1.0
        // strength = (9.0 + 1.5) * 1.0 = 10.5
        assert!((strength - 10.5).abs() < 0.01);
    }

    #[test]
    fn test_calc_strength_different_skills() {
        let player = create_test_player(8, 5, 10, false, 10, 12);
        
        let keeper_strength = calc_strength(&player, PlayerSkill::Keeper);
        let defender_strength = calc_strength(&player, PlayerSkill::Defending);
        
        // Defender skill (12) is higher than Keeper skill (10)
        // So defender strength should be higher
        assert!(defender_strength > keeper_strength);
        
        // Specifically: defender_rating = 11.0, keeper_rating = 9.0
        // Difference should be about 2.0 (scaled by form)
        assert!((defender_strength - keeper_strength - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_calc_player_tactic_strength() {
        // Form=8, Experience=7, Loyalty=10, Keeper=10
        let player = create_test_player(8, 7, 10, false, 10, 12);
        
        let tactic_strength = calc_player_tactic_strength(&player, PlayerSkill::Keeper);
        
        // loyalty ≈ 0.474
        // strength ≈ 9.474
        // xp = calc_skill_rating(7) = 6.0
        // f = log10(6.0) * 4.0 / 3.0 ≈ 0.778 * 1.333 ≈ 1.037
        // total ≈ 0.474 + 9.474 + 1.037 ≈ 10.985
        assert!((tactic_strength - 10.985).abs() < 0.1);
    }

    #[test]
    fn test_calc_player_tactic_strength_with_high_experience() {
        // High experience should increase tactic strength
        let player = create_test_player(8, 10, 10, false, 10, 12);
        
        let tactic_strength = calc_player_tactic_strength(&player, PlayerSkill::Keeper);
        
        // xp = calc_skill_rating(10) = 9.0
        // f = log10(9.0) * 4.0 / 3.0 ≈ 0.954 * 1.333 ≈ 1.272
        // This should be higher than with experience=7
        assert!(tactic_strength > 10.5);
    }

    #[test]
    fn test_get_player_skill_no_skills() {
        let mut player = create_test_player(8, 5, 10, false, 10, 12);
        player.PlayerSkills = None;
        
        // Should return 0 for all skills when PlayerSkills is None
        assert_eq!(get_player_skill(&player, PlayerSkill::Keeper), 0.0);
        assert_eq!(get_player_skill(&player, PlayerSkill::Defending), 0.0);
        assert_eq!(get_player_skill(&player, PlayerSkill::Playmaking), 0.0);
        
        // But Form, Experience, Loyalty should still work
        assert_eq!(get_player_skill(&player, PlayerSkill::Form), 8.0);
        assert_eq!(get_player_skill(&player, PlayerSkill::Experience), 5.0);
        assert_eq!(get_player_skill(&player, PlayerSkill::Loyalty), 10.0);
    }
}

