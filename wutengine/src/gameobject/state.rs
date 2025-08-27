use derive_more::IsVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, IsVariant)]
pub(crate) enum GameObjectState {
    /// Object disabled
    Disabled,

    /// Object enabled. Normal state
    Enabled,
}

impl core::fmt::Display for GameObjectState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GameObjectState::Disabled => write!(f, "Disabled"),
            GameObjectState::Enabled => write!(f, "Enabled"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IsVariant)]
pub(crate) enum GameObjectTargetState {
    /// Object disabled
    Disabled,

    /// Object enabled. Normal state
    Enabled,

    /// Object should be destroyed. Final. Once this state is
    /// requested, no other state request can override it
    Destroyed,
}

impl core::fmt::Display for GameObjectTargetState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GameObjectTargetState::Disabled => write!(f, "Disabled"),
            GameObjectTargetState::Enabled => write!(f, "Enabled"),
            GameObjectTargetState::Destroyed => write!(f, "Destroyed"),
        }
    }
}

impl From<GameObjectState> for GameObjectTargetState {
    fn from(value: GameObjectState) -> Self {
        match value {
            GameObjectState::Disabled => Self::Disabled,
            GameObjectState::Enabled => Self::Enabled,
        }
    }
}

impl TryFrom<GameObjectTargetState> for GameObjectState {
    type Error = ();

    fn try_from(value: GameObjectTargetState) -> Result<Self, Self::Error> {
        match value {
            GameObjectTargetState::Disabled => Ok(Self::Disabled),
            GameObjectTargetState::Enabled => Ok(Self::Enabled),
            GameObjectTargetState::Destroyed => Err(()),
        }
    }
}
