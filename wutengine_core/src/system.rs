#[derive(Debug)]
pub struct System<F> {
    pub phase: SystemPhase,
    pub func: F,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemPhase {
    RuntimeStart,
    Update,
}
