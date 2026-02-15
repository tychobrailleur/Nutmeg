/* metadata.rs
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

/// Metadata for a CHPP API endpoint
#[derive(Debug, Clone)]
pub struct EndpointInfo {
    /// Endpoint name (e.g., "teamdetails", "worlddetails")
    pub name: &'static str,
    /// Current API version
    pub version: &'static str,
    /// Human-readable description of what this endpoint provides
    pub description: &'static str,
    /// Link to official CHPP documentation
    pub documentation_url: &'static str,
}

/// Registry of all CHPP API endpoints with their versions and documentation
pub struct ChppEndpoints;

impl ChppEndpoints {
    pub const TEAM_DETAILS: EndpointInfo = EndpointInfo {
        name: "teamdetails",
        version: "3.8",
        description: "Team information",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=teamdetails",
    };

    pub const WORLD_DETAILS: EndpointInfo = EndpointInfo {
        name: "worlddetails",
        version: "1.9",
        description: "General Information about all countries in HT World",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=worlddetails",
    };

    pub const PLAYERS: EndpointInfo = EndpointInfo {
        name: "players",
        version: "2.8",
        description: "Players",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=players",
    };

    pub const PLAYER_DETAILS: EndpointInfo = EndpointInfo {
        name: "playerdetails",
        version: "3.2",
        description: "Detailed information for a player",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=playerdetails",
    };

    pub const MATCH_DETAILS: EndpointInfo = EndpointInfo {
        name: "matchdetails",
        version: "3.1",
        description: "Match details",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=matchdetails",
    };

    pub const MATCHES: EndpointInfo = EndpointInfo {
        name: "matches",
        version: "2.9",
        description: "List of matches",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=matches",
    };

    pub const ECONOMY: EndpointInfo = EndpointInfo {
        name: "economy",
        version: "1.4",
        description: "Team economy",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=economy",
    };

    pub const ARENA_DETAILS: EndpointInfo = EndpointInfo {
        name: "arenadetails",
        version: "1.7",
        description: "Arena information",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=arenadetails",
    };

    pub const TRAINING: EndpointInfo = EndpointInfo {
        name: "training",
        version: "2.2",
        description: "Training information",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=training",
    };

    pub const ACHIEVEMENTS: EndpointInfo = EndpointInfo {
        name: "achievements",
        version: "1.2",
        description: "The achievements of a specific user",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=achievements",
    };

    pub const ALLIANCE_DETAILS: EndpointInfo = EndpointInfo {
        name: "alliancedetails",
        version: "1.5",
        description: "Alliance / Federation information",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=alliancedetails",
    };

    pub const ALLIANCES: EndpointInfo = EndpointInfo {
        name: "alliances",
        version: "1.4",
        description: "Alliance / Federation search",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=alliances",
    };

    pub const AVATARS: EndpointInfo = EndpointInfo {
        name: "avatars",
        version: "1.1",
        description: "Avatars for all players of user's team",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=avatars",
    };

    pub const BOOKMARKS: EndpointInfo = EndpointInfo {
        name: "bookmarks",
        version: "1.0",
        description: "User bookmarks",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=bookmarks",
    };

    pub const CHALLENGES: EndpointInfo = EndpointInfo {
        name: "challenges",
        version: "1.6",
        description: "Challenges",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=challenges",
    };

    pub const CLUB: EndpointInfo = EndpointInfo {
        name: "club",
        version: "1.5",
        description: "Information about specialists and youth",
        documentation_url: "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=club",
    };

    pub const CUP_MATCHES: EndpointInfo = EndpointInfo {
        name: "cupmatches",
        version: "1.4",
        description: "Information about cup matches",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=cupmatches",
    };

    pub const CURRENT_BIDS: EndpointInfo = EndpointInfo {
        name: "currentbids",
        version: "1.0",
        description: "Shows the current transfer activity for a team",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=currentbids",
    };

    pub const FANS: EndpointInfo = EndpointInfo {
        name: "fans",
        version: "1.3",
        description: "Fanclub information",
        documentation_url: "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=fans",
    };

    pub const HOF_PLAYERS: EndpointInfo = EndpointInfo {
        name: "hofplayers",
        version: "1.2",
        description: "Hall of Fame Players",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=hofplayers",
    };

    pub const LADDER_DETAILS: EndpointInfo = EndpointInfo {
        name: "ladderdetails",
        version: "1.0",
        description: "Information about teams in the ladder and positions in it",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=ladderdetails",
    };

    pub const LADDER_LIST: EndpointInfo = EndpointInfo {
        name: "ladderlist",
        version: "1.0",
        description: "Information about ladder that the user is currently playing in",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=ladderlist",
    };

    pub const LEAGUE_DETAILS: EndpointInfo = EndpointInfo {
        name: "leaguedetails",
        version: "1.6",
        description: "Information about a League Level Unit (series)",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=leaguedetails",
    };

    pub const LEAGUE_FIXTURES: EndpointInfo = EndpointInfo {
        name: "leaguefixtures",
        version: "1.2",
        description: "Fixtures for a League Level Unit (series)",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=leaguefixtures",
    };

    pub const LEAGUE_LEVELS: EndpointInfo = EndpointInfo {
        name: "leaguelevels",
        version: "1.0",
        description: "Shows league level units (series) information for a specific league",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=leaguelevels",
    };

    pub const LIVE: EndpointInfo = EndpointInfo {
        name: "live",
        version: "2.3",
        description: "Get (live) match ticker",
        documentation_url: "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=live",
    };

    pub const MANAGER_COMPENDIUM: EndpointInfo = EndpointInfo {
        name: "managercompendium",
        version: "1.6",
        description: "The manager compendium of the logged in user",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=managercompendium",
    };

    pub const MATCHES_ARCHIVE: EndpointInfo = EndpointInfo {
        name: "matchesarchive",
        version: "1.5",
        description: "Matches Archive",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=matchesarchive",
    };

    pub const MATCH_ORDERS: EndpointInfo = EndpointInfo {
        name: "matchorders",
        version: "3.1",
        description: "Match orders for upcoming matches",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=matchorders",
    };

    pub const MATCH_LINEUP: EndpointInfo = EndpointInfo {
        name: "matchlineup",
        version: "2.1",
        description: "Match lineup for finished matches",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=matchlineup",
    };

    pub const PLAYER_EVENTS: EndpointInfo = EndpointInfo {
        name: "playerevents",
        version: "1.3",
        description: "Player events",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=playerevents",
    };

    pub const REGION_DETAILS: EndpointInfo = EndpointInfo {
        name: "regiondetails",
        version: "1.2",
        description: "Detailed information about a region",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=regiondetails",
    };

    pub const SEARCH: EndpointInfo = EndpointInfo {
        name: "search",
        version: "1.2",
        description: "Search",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=search",
    };

    pub const STAFF_AVATARS: EndpointInfo = EndpointInfo {
        name: "staffavatars",
        version: "1.1",
        description: "Avatars for all staff members",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=staffavatars",
    };

    pub const STAFF_LIST: EndpointInfo = EndpointInfo {
        name: "stafflist",
        version: "1.2",
        description: "A list of all staff members",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=stafflist",
    };

    pub const SUPPORTERS: EndpointInfo = EndpointInfo {
        name: "supporters",
        version: "1.0",
        description: "Information about teams supported and teams supporting",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=supporters",
    };

    pub const TOURNAMENT_DETAILS: EndpointInfo = EndpointInfo {
        name: "tournamentdetails",
        version: "1.0",
        description:
            "Information about a tournament. This is only available for the current season",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=tournamentdetails",
    };

    pub const TOURNAMENT_FIXTURES: EndpointInfo = EndpointInfo {
        name: "tournamentfixtures",
        version: "1.1",
        description: "Information about matches for a tournament",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=tournamentfixtures",
    };

    pub const TOURNAMENT_LEAGUE_TABLES: EndpointInfo = EndpointInfo {
        name: "tournamentleaguetables",
        version: "1.1",
        description: "League tables for a tournament",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=tournamentleaguetables",
    };

    pub const TOURNAMENT_LIST: EndpointInfo = EndpointInfo {
        name: "tournamentlist",
        version: "1.0",
        description: "Information about tournaments that the user is currently playing in",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=tournamentlist",
    };

    pub const TRAINING_EVENTS: EndpointInfo = EndpointInfo {
        name: "trainingevents",
        version: "1.3",
        description: "Get training events for a player",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=trainingevents",
    };

    pub const TRANSFER_SEARCH: EndpointInfo = EndpointInfo {
        name: "transfersearch",
        version: "1.1",
        description: "Search the transfer market",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=transfersearch",
    };

    pub const TRANSFERS_PLAYER: EndpointInfo = EndpointInfo {
        name: "transfersplayer",
        version: "1.1",
        description: "Get all transfers of a player",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=transfersplayer",
    };

    pub const TRANSFERS_TEAM: EndpointInfo = EndpointInfo {
        name: "transfersteam",
        version: "1.2",
        description: "Get the transfer history of a team",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=transfersteam",
    };

    pub const TRANSLATIONS: EndpointInfo = EndpointInfo {
        name: "translations",
        version: "1.2",
        description: "Translations for the denominations in the game",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=translations",
    };

    pub const WORLD_CUP: EndpointInfo = EndpointInfo {
        name: "worldcup",
        version: "1.1",
        description: "World cup groups and matches",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=worldcup",
    };

    pub const WORLD_LANGUAGES: EndpointInfo = EndpointInfo {
        name: "worldlanguages",
        version: "1.2",
        description: "Available languages",
        documentation_url:
            "https://www84.hattrick.org/Community/CHPP/NewDocs/File.aspx?name=worldlanguages",
    };

    /// Get all available endpoints
    pub fn all() -> Vec<EndpointInfo> {
        vec![
            Self::TEAM_DETAILS,
            Self::WORLD_DETAILS,
            Self::PLAYERS,
            Self::PLAYER_DETAILS,
            Self::MATCH_DETAILS,
            Self::MATCHES,
            Self::ECONOMY,
            Self::ARENA_DETAILS,
            Self::TRAINING,
            Self::ACHIEVEMENTS,
            Self::ALLIANCE_DETAILS,
            Self::ALLIANCES,
            Self::AVATARS,
            Self::BOOKMARKS,
            Self::CHALLENGES,
            Self::CLUB,
            Self::CUP_MATCHES,
            Self::CURRENT_BIDS,
            Self::FANS,
            Self::HOF_PLAYERS,
            Self::LADDER_DETAILS,
            Self::LADDER_LIST,
            Self::LEAGUE_DETAILS,
            Self::LEAGUE_FIXTURES,
            Self::LEAGUE_LEVELS,
            Self::LIVE,
            Self::MANAGER_COMPENDIUM,
            Self::MATCHES_ARCHIVE,
            Self::MATCH_ORDERS,
            Self::MATCH_LINEUP,
            Self::PLAYER_EVENTS,
            Self::REGION_DETAILS,
            Self::SEARCH,
            Self::STAFF_AVATARS,
            Self::STAFF_LIST,
            Self::SUPPORTERS,
            Self::TOURNAMENT_DETAILS,
            Self::TOURNAMENT_FIXTURES,
            Self::TOURNAMENT_LEAGUE_TABLES,
            Self::TOURNAMENT_LIST,
            Self::TRAINING_EVENTS,
            Self::TRANSFER_SEARCH,
            Self::TRANSFERS_PLAYER,
            Self::TRANSFERS_TEAM,
            Self::TRANSLATIONS,
            Self::WORLD_CUP,
            Self::WORLD_LANGUAGES,
        ]
    }

    /// Get endpoint info by name
    pub fn get_by_name(name: &str) -> Option<EndpointInfo> {
        Self::all().into_iter().find(|e| e.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_info_team_details() {
        let endpoint = ChppEndpoints::TEAM_DETAILS;
        assert_eq!(endpoint.name, "teamdetails");
        assert_eq!(endpoint.version, "3.8");
        assert!(endpoint.documentation_url.contains("teamdetails"));
        assert!(endpoint.documentation_url.starts_with("https://"));
    }

    #[test]
    fn test_endpoint_info_world_details() {
        let endpoint = ChppEndpoints::WORLD_DETAILS;
        assert_eq!(endpoint.name, "worlddetails");
        assert_eq!(endpoint.version, "1.9");
        assert!(endpoint.documentation_url.contains("worlddetails"));
    }

    #[test]
    fn test_endpoint_info_players() {
        let endpoint = ChppEndpoints::PLAYERS;
        assert_eq!(endpoint.name, "players");
        assert_eq!(endpoint.version, "2.8");
        assert!(endpoint.documentation_url.contains("players"));
    }

    #[test]
    fn test_endpoint_info_player_details() {
        let endpoint = ChppEndpoints::PLAYER_DETAILS;
        assert_eq!(endpoint.name, "playerdetails");
        assert_eq!(endpoint.version, "3.2");
        assert!(endpoint.documentation_url.contains("playerdetails"));
    }

    #[test]
    fn test_all_endpoints() {
        let endpoints = ChppEndpoints::all();
        assert_eq!(endpoints.len(), 47);

        // Verify core endpoints are present
        let names: Vec<&str> = endpoints.iter().map(|e| e.name).collect();
        assert!(names.contains(&"teamdetails"));
        assert!(names.contains(&"worlddetails"));
        assert!(names.contains(&"players"));
        assert!(names.contains(&"playerdetails"));
        assert!(names.contains(&"matchdetails"));
        assert!(names.contains(&"matches"));
        assert!(names.contains(&"economy"));
        assert!(names.contains(&"arenadetails"));
        assert!(names.contains(&"training"));

        // Verify some of the newly added endpoints
        assert!(names.contains(&"achievements"));
        assert!(names.contains(&"live"));
        assert!(names.contains(&"transfersearch"));
        assert!(names.contains(&"worldcup"));
    }

    #[test]
    fn test_get_by_name_found() {
        let endpoint = ChppEndpoints::get_by_name("worlddetails");
        assert!(endpoint.is_some());
        let endpoint = endpoint.unwrap();
        assert_eq!(endpoint.name, "worlddetails");
        assert_eq!(endpoint.version, "1.9");
    }

    #[test]
    fn test_get_by_name_not_found() {
        let endpoint = ChppEndpoints::get_by_name("nonexistent");
        assert!(endpoint.is_none());
    }

    #[test]
    fn test_endpoint_versions_are_valid() {
        // Ensure all versions follow semantic versioning pattern (X.Y)
        for endpoint in ChppEndpoints::all() {
            let parts: Vec<&str> = endpoint.version.split('.').collect();
            assert_eq!(parts.len(), 2, "Version should be in X.Y format");
            assert!(
                parts[0].parse::<u32>().is_ok(),
                "Major version should be a number"
            );
            assert!(
                parts[1].parse::<u32>().is_ok(),
                "Minor version should be a number"
            );
        }
    }

    #[test]
    fn test_all_documentation_urls_valid() {
        // Ensure all documentation URLs are valid HTTPS URLs
        for endpoint in ChppEndpoints::all() {
            assert!(endpoint.documentation_url.starts_with("https://"));
            assert!(endpoint.documentation_url.contains("hattrick.org"));
            assert!(endpoint.documentation_url.contains(endpoint.name));
        }
    }

    #[test]
    fn test_all_endpoints_have_descriptions() {
        // Ensure all endpoints have non-empty descriptions
        for endpoint in ChppEndpoints::all() {
            assert!(
                !endpoint.description.is_empty(),
                "Endpoint {} should have a description",
                endpoint.name
            );
            // Description should be a reasonable length
            assert!(
                endpoint.description.len() > 3,
                "Endpoint {} description seems too short",
                endpoint.name
            );
        }
    }

    #[test]
    fn test_endpoint_descriptions_content() {
        assert_eq!(ChppEndpoints::TEAM_DETAILS.description, "Team information");
        assert_eq!(
            ChppEndpoints::WORLD_DETAILS.description,
            "General Information about all countries in HT World"
        );
        assert_eq!(ChppEndpoints::PLAYERS.description, "Players");
        assert_eq!(
            ChppEndpoints::PLAYER_DETAILS.description,
            "Detailed information for a player"
        );
    }
}
