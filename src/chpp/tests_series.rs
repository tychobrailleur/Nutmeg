#[cfg(test)]
mod tests {
    use crate::chpp::model::{LeagueDetailsData, MatchesData};
    use serde_xml_rs::from_str;

    #[test]
    fn test_deserialize_league_details() {
        let xml = r#"
        <HattrickData>
            <LeagueLevelUnitID>100</LeagueLevelUnitID>
            <LeagueLevelUnitName>IV.10</LeagueLevelUnitName>
            <LeagueLevel>4</LeagueLevel>
            <MaxNumberOfTeams>8</MaxNumberOfTeams>
            <Team>
                <TeamID>1001</TeamID>
                <TeamName>Team A</TeamName>
                <Position>1</Position>
                <PositionChange>0</PositionChange>
                <Matches>5</Matches>
                <GoalsFor>10</GoalsFor>
                <GoalsAgainst>2</GoalsAgainst>
                <Points>15</Points>
                <Won>5</Won>
                <Draws>0</Draws>
                <Lost>0</Lost>
            </Team>
            <Team>
                <TeamID>1002</TeamID>
                <TeamName>Team B</TeamName>
                <Position>2</Position>
                <PositionChange>0</PositionChange>
                <Matches>5</Matches>
                <GoalsFor>8</GoalsFor>
                <GoalsAgainst>5</GoalsAgainst>
                <Points>12</Points>
                <Won>4</Won>
                <Draws>0</Draws>
                <Lost>1</Lost>
            </Team>
        </HattrickData>
        "#;

        let data: LeagueDetailsData =
            from_str(xml).expect("Failed to deserialize LeagueDetailsData");
        assert_eq!(data.LeagueLevelUnitID, 100);
        assert_eq!(data.LeagueLevelUnitName, "IV.10");
        assert_eq!(data.Teams.len(), 2);
        assert_eq!(data.Teams[0].TeamName, "Team A");
        assert_eq!(data.Teams[0].Points, 15);
    }

    #[test]
    fn test_deserialize_matches() {
        let xml = r#"<HattrickData><Team><TeamID>1001</TeamID><TeamName>My Team</TeamName><LeagueLevelUnitID>100</LeagueLevelUnitID><MatchList><Match><MatchID>5001</MatchID><HomeTeam><HomeTeamID>1001</HomeTeamID><HomeTeamName>My Team</HomeTeamName><HomeGoals>2</HomeGoals></HomeTeam><AwayTeam><AwayTeamID>1002</AwayTeamID><AwayTeamName>Opponent</AwayTeamName><AwayGoals>1</AwayGoals></AwayTeam><MatchDate>2023-11-01 15:00:00</MatchDate><MatchType>1</MatchType><Status>FINISHED</Status></Match><Match><MatchID>5002</MatchID><HomeTeam><HomeTeamID>1003</HomeTeamID><HomeTeamName>External</HomeTeamName></HomeTeam><AwayTeam><AwayTeamID>1001</AwayTeamID><AwayTeamName>My Team</AwayTeamName></AwayTeam><MatchDate>2023-11-08 15:00:00</MatchDate><MatchType>1</MatchType><Status>UPCOMING</Status></Match></MatchList></Team></HattrickData>"#;

        let data: MatchesData = from_str(xml).expect("Failed to deserialize MatchesData");
        assert_eq!(data.Team.TeamName, "My Team");
        assert_eq!(data.Team.MatchList.Matches.len(), 2);
        assert_eq!(data.Team.MatchList.Matches[0].HomeGoals, Some(2));
        assert_eq!(data.Team.MatchList.Matches[1].HomeGoals, None);
    }

    #[test]
    fn test_deserialize_matches_missing_list() {
        let xml = r#"<HattrickData><Team><TeamID>1001</TeamID><TeamName>My Team</TeamName><LeagueLevelUnitID>100</LeagueLevelUnitID></Team></HattrickData>"#;

        let data: MatchesData =
            from_str(xml).expect("Failed to deserialize MatchesData with missing MatchList");
        assert_eq!(data.Team.TeamName, "My Team");
        assert_eq!(data.Team.MatchList.Matches.len(), 0);
    }
}
