/// Core types and enums for the rating prediction system
/// Based on Schum's formula.

/// The seven rating sectors in a match
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RatingSector {
    DefenceLeft,
    DefenceCentral,
    DefenceRight,
    Midfield,
    AttackLeft,
    AttackCentral,
    AttackRight,
}

impl RatingSector {
    pub fn side(&self) -> Side {
        match self {
            Self::DefenceLeft | Self::AttackLeft => Side::Left,
            Self::DefenceCentral | Self::Midfield | Self::AttackCentral => Side::Middle,
            Self::DefenceRight | Self::AttackRight => Side::Right,
        }
    }

    pub fn all() -> [RatingSector; 7] {
        [
            Self::DefenceLeft,
            Self::DefenceCentral,
            Self::DefenceRight,
            Self::Midfield,
            Self::AttackLeft,
            Self::AttackCentral,
            Self::AttackRight,
        ]
    }
}

/// Side of the field
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Left,
    Middle,
    Right,
}

/// Contribution parameter sets for different rating calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RatingContributionParameterSet {
    SideDefence,
    CentralDefence,
    Midfield,
    SideAttack,
    CentralAttack,
}

/// Side restrictions for position contributions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SideRestriction {
    None,             // All sides contribute to the ratings
    ThisSideOnly,     // Only this side contributes
    MiddleOnly,       // Only middle positions contribute
    OppositeSideOnly, // Only opposite side contributes
}

/// Player behavior/orientation on the field
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Behaviour {
    Normal = 0,
    Offensive = 1,
    Defensive = 2,
    TowardsMiddle = 3,
    TowardsWing = 4,
}

impl From<u8> for Behaviour {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::Offensive,
            2 => Self::Defensive,
            3 => Self::TowardsMiddle,
            4 => Self::TowardsWing,
            _ => Self::Normal,
        }
    }
}

impl Behaviour {
    /// Returns a Unicode symbol representing the behaviour direction.
    pub fn symbol(&self, position: &PositionId) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Offensive => "▲",
            Self::Defensive => "▼",
            Self::TowardsMiddle => {
                if position.is_left_side() {
                    "▶"
                } else if position.is_right_side() {
                    "◀"
                } else {
                    "↔"
                }
            }
            Self::TowardsWing => {
                if position.is_left_side() {
                    "◀"
                } else if position.is_right_side() {
                    "▶"
                } else {
                    "↔"
                }
            }
        }
    }

    /// Returns a human-readable name for display.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Offensive => "Offensive",
            Self::Defensive => "Defensive",
            Self::TowardsMiddle => "Towards Middle",
            Self::TowardsWing => "Towards Wing",
        }
    }
}

/// Lineup sectors (positions on the field)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sector {
    Goal,
    CentralDefence,
    Back,
    InnerMidfield,
    Wing,
    Forward,
}

/// Position IDs for all field positions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PositionId {
    Keeper = 100,

    // Defence
    LeftBack = 101,
    LeftCentralDefender = 102,
    MiddleCentralDefender = 103,
    RightCentralDefender = 104,
    RightBack = 105,

    // Midfield
    LeftWinger = 106,
    LeftInnerMidfield = 107,
    CentralInnerMidfield = 108,
    RightInnerMidfield = 109,
    RightWinger = 110,

    // Attack
    LeftForward = 111,
    CentralForward = 112,
    RightForward = 113,

    // Special
    SetPieces = 17,
}

impl PositionId {
    pub fn sector(&self) -> Sector {
        match self {
            Self::Keeper => Sector::Goal,
            Self::LeftCentralDefender
            | Self::MiddleCentralDefender
            | Self::RightCentralDefender => Sector::CentralDefence,
            Self::LeftBack | Self::RightBack => Sector::Back,
            Self::LeftInnerMidfield | Self::CentralInnerMidfield | Self::RightInnerMidfield => {
                Sector::InnerMidfield
            }
            Self::LeftWinger | Self::RightWinger => Sector::Wing,
            Self::LeftForward | Self::CentralForward | Self::RightForward => Sector::Forward,
            Self::SetPieces => Sector::InnerMidfield, // Approximation
        }
    }

    pub fn is_left_side(&self) -> bool {
        matches!(
            self,
            Self::LeftBack
                | Self::LeftCentralDefender
                | Self::LeftInnerMidfield
                | Self::LeftWinger
                | Self::LeftForward
        )
    }

    pub fn is_right_side(&self) -> bool {
        matches!(
            self,
            Self::RightBack
                | Self::RightCentralDefender
                | Self::RightInnerMidfield
                | Self::RightWinger
                | Self::RightForward
        )
    }

    pub fn is_middle(&self) -> bool {
        matches!(
            self,
            Self::Keeper
                | Self::MiddleCentralDefender
                | Self::CentralInnerMidfield
                | Self::CentralForward
        )
    }

    /// Returns the valid behaviours for this position.
    /// Keepers only have Normal. Wingers cannot go "towards wing".
    /// Central positions cannot go "towards middle", etc.
    pub fn valid_behaviours(&self) -> Vec<Behaviour> {
        match self.sector() {
            Sector::Goal => vec![Behaviour::Normal],
            Sector::CentralDefence => {
                vec![
                    Behaviour::Normal,
                    Behaviour::Offensive,
                    Behaviour::TowardsWing,
                ]
            }
            Sector::Back => {
                vec![
                    Behaviour::Normal,
                    Behaviour::Offensive,
                    Behaviour::Defensive,
                    Behaviour::TowardsMiddle,
                ]
            }
            Sector::InnerMidfield => {
                if self.is_middle() {
                    vec![
                        Behaviour::Normal,
                        Behaviour::Offensive,
                        Behaviour::Defensive,
                        Behaviour::TowardsWing,
                    ]
                } else {
                    vec![
                        Behaviour::Normal,
                        Behaviour::Offensive,
                        Behaviour::Defensive,
                        Behaviour::TowardsMiddle,
                        Behaviour::TowardsWing,
                    ]
                }
            }
            Sector::Wing => {
                vec![
                    Behaviour::Normal,
                    Behaviour::Offensive,
                    Behaviour::Defensive,
                    Behaviour::TowardsMiddle,
                ]
            }
            Sector::Forward => {
                if self.is_middle() {
                    vec![
                        Behaviour::Normal,
                        Behaviour::Defensive,
                        Behaviour::TowardsWing,
                    ]
                } else {
                    vec![
                        Behaviour::Normal,
                        Behaviour::Defensive,
                        Behaviour::TowardsMiddle,
                    ]
                }
            }
        }
    }
}

/// Player skill types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerSkill {
    Keeper,
    Defending,
    Playmaking,
    Passing,
    Winger,
    Scoring,
    SetPieces,
    Form,
    Stamina,
    Experience,
    Loyalty,
}

/// Player specialty types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[allow(clippy::enum_variant_names)] // NoSpecialty is the Hattrick domain term for specialty code 0
pub enum Specialty {
    #[default]
    NoSpecialty,
    Technical,
    Quick,
    Powerful,
    Unpredictable,
    Head,
    Regainer,
    Support,
}

/// Weather conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Weather {
    #[default]
    Neutral,
    Rainy,
    Sunny,
}

/// Match tactic types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TacticType {
    #[default]
    Normal = 0,
    Pressing = 1,
    CounterAttacks = 2,
    AttackInTheMiddle = 3,
    AttackOnWings = 4,
    PlayCreatively = 7,
    LongShots = 8,
}

/// Match attitude
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Attitude {
    #[default]
    Normal = 0,
    PlayItCool = 1,       // PIC
    MatchOfTheSeason = 2, // MOTS
}

/// Match location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Location {
    #[default]
    Away = 0,
    Home = 1,
    AwayDerby = 2,
}
