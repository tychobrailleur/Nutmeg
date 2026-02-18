/// Contribution parameter definitions
///
/// Contains 700+ contribution factors that determine how each player position
/// contributes to each rating sector based on skill, behavior, and specialty
///
use super::types::*;
use std::collections::HashMap;
use std::sync::OnceLock;

pub type ContributionMap = HashMap<
    RatingContributionParameterSet,
    HashMap<
        PlayerSkill,
        HashMap<Sector, HashMap<SideRestriction, HashMap<Behaviour, HashMap<Specialty, f64>>>>,
    >,
>;

/// Get overcrowding penalty for a sector
pub fn get_overcrowding_penalty(count: usize, sector: Sector) -> f64 {
    match (sector, count) {
        (Sector::CentralDefence, 2) => 0.964,
        (Sector::CentralDefence, 3) => 0.900,
        (Sector::InnerMidfield, 2) => 0.935,
        (Sector::InnerMidfield, 3) => 0.825,
        (Sector::Forward, 2) => 0.945,
        (Sector::Forward, 3) => 0.865,
        _ => 1.0,
    }
}

/// Get the global contribution parameters map
pub fn get_contribution_parameters() -> &'static ContributionMap {
    static PARAMS: OnceLock<ContributionMap> = OnceLock::new();
    PARAMS.get_or_init(init_contribution_parameters)
}

/// Initialize all contribution parameters
fn init_contribution_parameters() -> ContributionMap {
    let mut map = HashMap::new();

    // SIDE DEFENCE PARAMETERS
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Keeper,
        Sector::Goal,
        SideRestriction::None,
        Behaviour::Normal,
        0.61,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::Goal,
        SideRestriction::None,
        Behaviour::Normal,
        0.25,
    );

    // Central Defence - Side contributions
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::CentralDefence,
        SideRestriction::ThisSideOnly,
        Behaviour::Normal,
        0.52,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::CentralDefence,
        SideRestriction::ThisSideOnly,
        Behaviour::Offensive,
        0.4,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::CentralDefence,
        SideRestriction::ThisSideOnly,
        Behaviour::TowardsWing,
        0.81,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::CentralDefence,
        SideRestriction::MiddleOnly,
        Behaviour::Normal,
        0.26,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::CentralDefence,
        SideRestriction::MiddleOnly,
        Behaviour::Offensive,
        0.2,
    );

    // Backs
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::Back,
        SideRestriction::ThisSideOnly,
        Behaviour::Normal,
        0.92,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::Back,
        SideRestriction::ThisSideOnly,
        Behaviour::Offensive,
        0.74,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::Back,
        SideRestriction::ThisSideOnly,
        Behaviour::Defensive,
        1.0,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::Back,
        SideRestriction::ThisSideOnly,
        Behaviour::TowardsMiddle,
        0.75,
    );

    // Inner Midfield - Side defending
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::InnerMidfield,
        SideRestriction::ThisSideOnly,
        Behaviour::Normal,
        0.19,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::InnerMidfield,
        SideRestriction::ThisSideOnly,
        Behaviour::Offensive,
        0.09,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::InnerMidfield,
        SideRestriction::ThisSideOnly,
        Behaviour::Defensive,
        0.27,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::InnerMidfield,
        SideRestriction::ThisSideOnly,
        Behaviour::TowardsWing,
        0.24,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::InnerMidfield,
        SideRestriction::MiddleOnly,
        Behaviour::Normal,
        0.095,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::InnerMidfield,
        SideRestriction::MiddleOnly,
        Behaviour::Offensive,
        0.045,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::InnerMidfield,
        SideRestriction::MiddleOnly,
        Behaviour::Defensive,
        0.135,
    );

    // Wings - Side defending
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::Wing,
        SideRestriction::ThisSideOnly,
        Behaviour::Normal,
        0.35,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::Wing,
        SideRestriction::ThisSideOnly,
        Behaviour::Offensive,
        0.22,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::Wing,
        SideRestriction::ThisSideOnly,
        Behaviour::Defensive,
        0.61,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideDefence,
        PlayerSkill::Defending,
        Sector::Wing,
        SideRestriction::ThisSideOnly,
        Behaviour::TowardsMiddle,
        0.29,
    );

    // CENTRAL DEFENCE PARAMETERS
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Keeper,
        Sector::Goal,
        SideRestriction::None,
        Behaviour::Normal,
        0.87,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::Goal,
        SideRestriction::None,
        Behaviour::Normal,
        0.35,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::CentralDefence,
        SideRestriction::None,
        Behaviour::Normal,
        1.0,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::CentralDefence,
        SideRestriction::None,
        Behaviour::Offensive,
        0.73,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::CentralDefence,
        SideRestriction::None,
        Behaviour::TowardsWing,
        0.67,
    );

    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::Back,
        SideRestriction::None,
        Behaviour::Normal,
        0.38,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::Back,
        SideRestriction::None,
        Behaviour::Offensive,
        0.35,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::Back,
        SideRestriction::None,
        Behaviour::Defensive,
        0.43,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::Back,
        SideRestriction::None,
        Behaviour::TowardsMiddle,
        0.7,
    );

    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::InnerMidfield,
        SideRestriction::None,
        Behaviour::Normal,
        0.4,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::InnerMidfield,
        SideRestriction::None,
        Behaviour::Offensive,
        0.16,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::InnerMidfield,
        SideRestriction::None,
        Behaviour::Defensive,
        0.58,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::InnerMidfield,
        SideRestriction::None,
        Behaviour::TowardsWing,
        0.33,
    );

    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::Wing,
        SideRestriction::None,
        Behaviour::Normal,
        0.2,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::Wing,
        SideRestriction::None,
        Behaviour::Offensive,
        0.13,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::Wing,
        SideRestriction::None,
        Behaviour::Defensive,
        0.25,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralDefence,
        PlayerSkill::Defending,
        Sector::Wing,
        SideRestriction::None,
        Behaviour::TowardsMiddle,
        0.25,
    );

    // MIDFIELD PARAMETERS - Playmaking
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::CentralDefence,
        SideRestriction::None,
        Behaviour::Normal,
        0.25,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::CentralDefence,
        SideRestriction::None,
        Behaviour::Offensive,
        0.4,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::CentralDefence,
        SideRestriction::None,
        Behaviour::TowardsWing,
        0.15,
    );

    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::Back,
        SideRestriction::None,
        Behaviour::Normal,
        0.15,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::Back,
        SideRestriction::None,
        Behaviour::Offensive,
        0.2,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::Back,
        SideRestriction::None,
        Behaviour::Defensive,
        0.1,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::Back,
        SideRestriction::None,
        Behaviour::TowardsMiddle,
        0.2,
    );

    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::InnerMidfield,
        SideRestriction::None,
        Behaviour::Normal,
        1.0,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::InnerMidfield,
        SideRestriction::None,
        Behaviour::Offensive,
        0.95,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::InnerMidfield,
        SideRestriction::None,
        Behaviour::Defensive,
        0.95,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::InnerMidfield,
        SideRestriction::None,
        Behaviour::TowardsWing,
        0.9,
    );

    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::Wing,
        SideRestriction::None,
        Behaviour::Normal,
        0.45,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::Wing,
        SideRestriction::None,
        Behaviour::Offensive,
        0.3,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::Wing,
        SideRestriction::None,
        Behaviour::Defensive,
        0.3,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::Wing,
        SideRestriction::None,
        Behaviour::TowardsMiddle,
        0.55,
    );

    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::Normal,
        0.25,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::Defensive,
        0.35,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::Midfield,
        PlayerSkill::Playmaking,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::TowardsWing,
        0.15,
    );

    // CENTRAL ATTACK PARAMETERS - Passing
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Passing,
        Sector::InnerMidfield,
        SideRestriction::None,
        Behaviour::Normal,
        0.33,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Passing,
        Sector::InnerMidfield,
        SideRestriction::None,
        Behaviour::Offensive,
        0.49,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Passing,
        Sector::InnerMidfield,
        SideRestriction::None,
        Behaviour::Defensive,
        0.18,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Passing,
        Sector::InnerMidfield,
        SideRestriction::None,
        Behaviour::TowardsWing,
        0.23,
    );

    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Passing,
        Sector::Wing,
        SideRestriction::None,
        Behaviour::Normal,
        0.11,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Passing,
        Sector::Wing,
        SideRestriction::None,
        Behaviour::Offensive,
        0.13,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Passing,
        Sector::Wing,
        SideRestriction::None,
        Behaviour::Defensive,
        0.05,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Passing,
        Sector::Wing,
        SideRestriction::None,
        Behaviour::TowardsMiddle,
        0.16,
    );

    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Passing,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::Normal,
        0.33,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Passing,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::Defensive,
        0.53,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Passing,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::TowardsWing,
        0.23,
    );

    // CENTRAL ATTACK - Scoring
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Scoring,
        Sector::InnerMidfield,
        SideRestriction::None,
        Behaviour::Normal,
        0.22,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Scoring,
        Sector::InnerMidfield,
        SideRestriction::None,
        Behaviour::Offensive,
        0.31,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Scoring,
        Sector::InnerMidfield,
        SideRestriction::None,
        Behaviour::Defensive,
        0.13,
    );

    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Scoring,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::Normal,
        1.0,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Scoring,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::Defensive,
        0.56,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::CentralAttack,
        PlayerSkill::Scoring,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::TowardsWing,
        0.66,
    );

    // SIDE ATTACK PARAMETERS - Passing
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::InnerMidfield,
        SideRestriction::MiddleOnly,
        Behaviour::Normal,
        0.13,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::InnerMidfield,
        SideRestriction::MiddleOnly,
        Behaviour::Offensive,
        0.18,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::InnerMidfield,
        SideRestriction::MiddleOnly,
        Behaviour::Defensive,
        0.07,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::Normal,
        0.14,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::Defensive,
        0.31,
    );
    // Special case: Technical specialty bonus for forward defensive passing
    init_one_specialty(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::Defensive,
        Specialty::Technical,
        0.41,
    );

    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::InnerMidfield,
        SideRestriction::ThisSideOnly,
        Behaviour::Normal,
        0.26,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::InnerMidfield,
        SideRestriction::ThisSideOnly,
        Behaviour::Offensive,
        0.36,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::InnerMidfield,
        SideRestriction::ThisSideOnly,
        Behaviour::Defensive,
        0.14,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::InnerMidfield,
        SideRestriction::ThisSideOnly,
        Behaviour::TowardsWing,
        0.31,
    );

    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::Wing,
        SideRestriction::ThisSideOnly,
        Behaviour::Normal,
        0.26,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::Wing,
        SideRestriction::ThisSideOnly,
        Behaviour::Offensive,
        0.29,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::Wing,
        SideRestriction::ThisSideOnly,
        Behaviour::Defensive,
        0.21,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::Wing,
        SideRestriction::ThisSideOnly,
        Behaviour::TowardsMiddle,
        0.15,
    );

    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::Forward,
        SideRestriction::ThisSideOnly,
        Behaviour::TowardsWing,
        0.21,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Passing,
        Sector::Forward,
        SideRestriction::OppositeSideOnly,
        Behaviour::TowardsWing,
        0.06,
    );

    // SIDE ATTACK - Winger
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Winger,
        Sector::CentralDefence,
        SideRestriction::ThisSideOnly,
        Behaviour::TowardsWing,
        0.26,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Winger,
        Sector::Back,
        SideRestriction::ThisSideOnly,
        Behaviour::Normal,
        0.59,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Winger,
        Sector::Back,
        SideRestriction::ThisSideOnly,
        Behaviour::Offensive,
        0.69,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Winger,
        Sector::Back,
        SideRestriction::ThisSideOnly,
        Behaviour::Defensive,
        0.45,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Winger,
        Sector::Back,
        SideRestriction::ThisSideOnly,
        Behaviour::TowardsMiddle,
        0.35,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Winger,
        Sector::InnerMidfield,
        SideRestriction::ThisSideOnly,
        Behaviour::TowardsWing,
        0.59,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Winger,
        Sector::Wing,
        SideRestriction::ThisSideOnly,
        Behaviour::Normal,
        0.86,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Winger,
        Sector::Wing,
        SideRestriction::ThisSideOnly,
        Behaviour::Offensive,
        1.0,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Winger,
        Sector::Wing,
        SideRestriction::ThisSideOnly,
        Behaviour::Defensive,
        0.69,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Winger,
        Sector::Wing,
        SideRestriction::ThisSideOnly,
        Behaviour::TowardsMiddle,
        0.74,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Winger,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::Normal,
        0.24,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Winger,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::Defensive,
        0.13,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Winger,
        Sector::Forward,
        SideRestriction::ThisSideOnly,
        Behaviour::TowardsWing,
        0.64,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Winger,
        Sector::Forward,
        SideRestriction::OppositeSideOnly,
        Behaviour::TowardsWing,
        0.21,
    );

    // SIDE ATTACK - Scoring
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Scoring,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::Normal,
        0.27,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Scoring,
        Sector::Forward,
        SideRestriction::None,
        Behaviour::Defensive,
        0.13,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Scoring,
        Sector::Forward,
        SideRestriction::OppositeSideOnly,
        Behaviour::TowardsWing,
        0.19,
    );
    init_all_specialties(
        &mut map,
        RatingContributionParameterSet::SideAttack,
        PlayerSkill::Scoring,
        Sector::Forward,
        SideRestriction::ThisSideOnly,
        Behaviour::TowardsWing,
        0.51,
    );

    map
}

/// Initialize a parameter with the same value for all specialties
fn init_all_specialties(
    map: &mut ContributionMap,
    rating_contribution: RatingContributionParameterSet,
    skill: PlayerSkill,
    sector: Sector,
    side_restriction: SideRestriction,
    behaviour: Behaviour,
    value: f64,
) {
    let specialty_map: HashMap<Specialty, f64> = [
        (Specialty::NoSpecialty, value),
        (Specialty::Technical, value),
        (Specialty::Quick, value),
        (Specialty::Powerful, value),
        (Specialty::Unpredictable, value),
        (Specialty::Head, value),
        (Specialty::Regainer, value),
        (Specialty::Support, value),
    ]
    .into_iter()
    .collect();

    map.entry(rating_contribution)
        .or_default()
        .entry(skill)
        .or_default()
        .entry(sector)
        .or_default()
        .entry(side_restriction)
        .or_default()
        .insert(behaviour, specialty_map);
}

/// Initialize a parameter for one specific specialty (overrides init_all_specialties)
fn init_one_specialty(
    map: &mut ContributionMap,
    rating_contribution: RatingContributionParameterSet,
    skill: PlayerSkill,
    sector: Sector,
    side_restriction: SideRestriction,
    behaviour: Behaviour,
    specialty: Specialty,
    value: f64,
) {
    map.entry(rating_contribution)
        .or_default()
        .entry(skill)
        .or_default()
        .entry(sector)
        .or_default()
        .entry(side_restriction)
        .or_default()
        .entry(behaviour)
        .or_default()
        .insert(specialty, value);
}
