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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Specialty {
    NoSpecialty,
    Technical,
    Quick,
    Powerful,
    Unpredictable,
    Head,
    Regainer,
    Support,
}

impl Default for Specialty {
    fn default() -> Self {
        Self::NoSpecialty
    }
}

/// Weather conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weather {
    Neutral,
    Rainy,
    Sunny,
}

impl Default for Weather {
    fn default() -> Self {
        Self::Neutral
    }
}

/// Match tactic types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TacticType {
    Normal = 0,
    Pressing = 1,
    CounterAttacks = 2,
    AttackInTheMiddle = 3,
    AttackOnWings = 4,
    PlayCreatively = 7,
    LongShots = 8,
}

impl Default for TacticType {
    fn default() -> Self {
        Self::Normal
    }
}

/// Match attitude
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Attitude {
    Normal = 0,
    PlayItCool = 1,       // PIC
    MatchOfTheSeason = 2, // MOTS
}

impl Default for Attitude {
    fn default() -> Self {
        Self::Normal
    }
}

/// Match location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Location {
    Away = 0,
    Home = 1,
    AwayDerby = 2,
}

impl Default for Location {
    fn default() -> Self {
        Self::Away
    }
}
