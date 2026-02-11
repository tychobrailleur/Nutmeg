use super::parameters::get_contribution_parameters;
use super::strength::calc_strength;
/// Contribution calculations for player positions to rating sectors
use super::types::*;
use crate::chpp::model::Player;

/// Calculate player's contribution to a rating sector
///
/// Looks up the appropriate contribution factors based on:
/// - Player's position and behaviour
/// - Rating sector being calculated
/// - Player's specialty
///
/// Returns the total contribution value (strength × factor)
pub fn calc_contribution(
    player: &Player,
    role_id: PositionId,
    behaviour: Behaviour,
    sector: RatingSector,
) -> f64 {
    let params = get_contribution_parameters();
    let player_sector = role_id.sector();
    let player_specialty = get_player_specialty(player);

    // Determine which rating contribution parameter set to use
    let rating_param_set = match sector {
        RatingSector::DefenceLeft | RatingSector::DefenceRight => {
            RatingContributionParameterSet::SideDefence
        }
        RatingSector::DefenceCentral => RatingContributionParameterSet::CentralDefence,
        RatingSector::Midfield => RatingContributionParameterSet::Midfield,
        RatingSector::AttackLeft | RatingSector::AttackRight => {
            RatingContributionParameterSet::SideAttack
        }
        RatingSector::AttackCentral => RatingContributionParameterSet::CentralAttack,
    };

    // Get the parameters for this rating type
    let Some(skill_params) = params.get(&rating_param_set) else {
        return 0.0;
    };

    let mut total_contribution = 0.0;

    // Iterate through all skills that contribute to this rating sector
    for (skill, sector_params) in skill_params.iter() {
        // Check if this player's position contributes
        let Some(side_params) = sector_params.get(&player_sector) else {
            continue;
        };

        // Check all side restrictions
        for (side_restriction, behaviour_params) in side_params.iter() {
            // Check if side restriction blocks this position from contributing
            if is_role_side_restricted(role_id, sector.side(), *side_restriction) {
                continue;
            }

            // Get the behavior parameters
            let Some(specialty_params) = behaviour_params.get(&behaviour) else {
                continue;
            };

            // Get the factor for this specialty (or use NoSpecialty as default)
            let factor = specialty_params
                .get(&player_specialty)
                .or_else(|| specialty_params.get(&Specialty::NoSpecialty))
                .copied()
                .unwrap_or(0.0);

            if factor > 0.0 {
                // Calculate player strength for this skill and multiply by factor
                let strength = calc_strength(player, *skill);
                total_contribution += strength * factor;
            }
        }
    }

    total_contribution
}

/// Get player's specialty
fn get_player_specialty(player: &Player) -> Specialty {
    match player.Speciality {
        Some(1) => Specialty::Technical,
        Some(2) => Specialty::Quick,
        Some(3) => Specialty::Powerful,
        Some(4) => Specialty::Unpredictable,
        Some(5) => Specialty::Head,
        Some(6) => Specialty::Regainer,
        Some(8) => Specialty::Support,
        _ => Specialty::NoSpecialty,
    }
}

/// Check if a role is restricted from contributing to a given side
fn is_role_side_restricted(
    role_id: PositionId,
    side: Side,
    side_restriction: SideRestriction,
) -> bool {
    match side_restriction {
        SideRestriction::None => false,
        SideRestriction::MiddleOnly => !matches!(
            role_id,
            PositionId::Keeper
                | PositionId::MiddleCentralDefender
                | PositionId::CentralInnerMidfield
                | PositionId::CentralForward
        ),
        SideRestriction::ThisSideOnly => match side {
            Side::Left => !matches!(
                role_id,
                PositionId::LeftBack
                    | PositionId::LeftCentralDefender
                    | PositionId::LeftWinger
                    | PositionId::LeftInnerMidfield
                    | PositionId::LeftForward
            ),
            Side::Right => !matches!(
                role_id,
                PositionId::RightBack
                    | PositionId::RightCentralDefender
                    | PositionId::RightWinger
                    | PositionId::RightInnerMidfield
                    | PositionId::RightForward
            ),
            Side::Middle => !matches!(
                role_id,
                PositionId::Keeper
                    | PositionId::MiddleCentralDefender
                    | PositionId::CentralInnerMidfield
                    | PositionId::CentralForward
            ),
        },
        SideRestriction::OppositeSideOnly => match side {
            Side::Right => !matches!(
                role_id,
                PositionId::LeftBack
                    | PositionId::LeftCentralDefender
                    | PositionId::LeftWinger
                    | PositionId::LeftInnerMidfield
                    | PositionId::LeftForward
            ),
            Side::Left => !matches!(
                role_id,
                PositionId::RightBack
                    | PositionId::RightCentralDefender
                    | PositionId::RightWinger
                    | PositionId::RightInnerMidfield
                    | PositionId::RightForward
            ),
            Side::Middle => false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_side_restriction_middle_only() {
        assert!(!is_role_side_restricted(
            PositionId::CentralInnerMidfield,
            Side::Middle,
            SideRestriction::MiddleOnly
        ));
        assert!(is_role_side_restricted(
            PositionId::LeftInnerMidfield,
            Side::Left,
            SideRestriction::MiddleOnly
        ));
    }

    #[test]
    fn test_side_restriction_this_side_only() {
        assert!(!is_role_side_restricted(
            PositionId::LeftBack,
            Side::Left,
            SideRestriction::ThisSideOnly
        ));
        assert!(is_role_side_restricted(
            PositionId::RightBack,
            Side::Left,
            SideRestriction::ThisSideOnly
        ));
    }

    // Commented out - requires Player::default() which doesn't exist
    // #[test]
    // fn test_get_player_specialty() {
    //     let mut player = Player::default();
    //     assert_eq!(get_player_specialty(&player), Specialty::NoSpecialty);
    //
    //     player.Specialty = Some(1);
    //     assert_eq!(get_player_specialty(&player), Specialty::Technical);
    //
    //     player.Specialty = Some(2);
    //     assert_eq!(get_player_specialty(&player), Specialty::Quick);
    // }

    #[test]
    fn test_side_restriction_keeper_middle_only() {
        // Critical: Keeper should be allowed for MiddleOnly
        assert!(
            !is_role_side_restricted(
                PositionId::Keeper,
                Side::Middle,
                SideRestriction::MiddleOnly
            ),
            "Keeper should NOT be blocked by MiddleOnly restriction"
        );
    }

    #[test]
    fn test_side_restriction_keeper_this_side_middle() {
        // Critical: Keeper should be allowed for Middle ThisSideOnly
        assert!(
            !is_role_side_restricted(
                PositionId::Keeper,
                Side::Middle,
                SideRestriction::ThisSideOnly
            ),
            "Keeper should be allowed for Middle ThisSideOnly"
        );
    }
}
