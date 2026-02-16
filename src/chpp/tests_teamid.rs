use crate::chpp::model::Team;

#[test]
fn test_team_id_json_integer() {
    // Test that TeamID can be deserialized from JSON with integer value (old DB records)
    let json = r#"{
        "TeamID": 1498278,
        "TeamName": "Test Team",
        "ShortTeamName": null,
        "IsPrimaryClub": true,
        "FoundedDate": "2000-01-01",
        "ArenaID": 123,
        "ArenaName": "Test Arena",
        "HomePage": "https://example.com",
        "LeagueLevelUnitID": 456,
        "LeagueLevelUnitName": "IV.32",
        "LeagueID": 789,
        "LeagueName": "Test League",
        "SupporterTier": "none",
        "LogoURL": null,
        "NumberOfVictories": 0,
        "NumberOfUndefeated": 0,
        "IsYouth": false
    }"#;
    let result: Result<Team, _> = serde_json::from_str(json);
    assert!(
        result.is_ok(),
        "Failed to deserialize Team with integer TeamID from JSON: {:?}",
        result.err()
    );
    let team = result.unwrap();
    assert_eq!(team.TeamID, "1498278");
}

#[test]
fn test_team_id_json_string() {
    // Test that TeamID can be deserialized from JSON with string value
    let json = r#"{
        "TeamID": "1498278",
        "TeamName": "Test Team",
        "ShortTeamName": null,
        "IsPrimaryClub": true,
        "FoundedDate": "2000-01-01",
        "ArenaID": 123,
        "ArenaName": "Test Arena",
        "HomePage": "https://example.com",
        "LeagueLevelUnitID": 456,
        "LeagueLevelUnitName": "IV.32",
        "LeagueID": 789,
        "LeagueName": "Test League",
        "SupporterTier": "none",
        "LogoURL": null,
        "NumberOfVictories": 0,
        "NumberOfUndefeated": 0,
        "IsYouth": false
    }"#;
    let result: Result<Team, _> = serde_json::from_str(json);
    assert!(
        result.is_ok(),
        "Failed to deserialize Team with string TeamID from JSON: {:?}",
        result.err()
    );
    let team = result.unwrap();
    assert_eq!(team.TeamID, "1498278");
}
