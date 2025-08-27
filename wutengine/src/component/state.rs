use derive_more::IsVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, IsVariant)]
pub(crate) enum ComponentState {
    /// Component disabled and not queued for activation
    Disabled,

    /// Component enabled. Normal state
    Enabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IsVariant)]
pub(crate) enum ComponentTargetState {
    /// Component should be disabled
    Disabled,

    /// Component should be enabled
    Enabled,

    /// Component should be destroyed. Final. Cannot be overridden again
    Destroyed,
}
