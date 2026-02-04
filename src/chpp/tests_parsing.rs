mod tests_xml_parsing {

    #[test]
    fn test_parse_world_details_xml() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("worlddetails2.xml");

        let xml_content = fs::read_to_string(d).expect("Failed to read worlddetails2.xml");

        // Attempt to deserialize
        let world_details: Result<WorldDetails, _> = serde_xml_rs::from_str(&xml_content);

        assert!(
            world_details.is_ok(),
            "Failed to parse XML: {:?}",
            world_details.err()
        );
        let details = world_details.unwrap();

        // Basic verification
        assert!(details.LeagueList.Leagues.len() > 0, "Should have leagues");

        // Verify specific League data (e.g., LeagueID 1 from the file content view)
        if let Some(league_sweden) = details.LeagueList.Leagues.iter().find(|l| l.LeagueID == 1) {
            assert_eq!(league_sweden.LeagueName, "Sweden");
            assert_eq!(league_sweden.SeasonOffset, Some(0));
            assert_eq!(league_sweden.Season, Some(93));
            assert_eq!(league_sweden.Country.CountryID, Some(1));
            assert_eq!(
                league_sweden.Country.CountryName,
                Some("Sverige".to_string())
            );
            assert_eq!(league_sweden.Country.CountryCode, Some("SE".to_string()));
        } else {
            panic!("LeagueID 1 not found");
        }
    }
}
