#[derive(Debug)]
pub struct System<I, O> {
    pub phase: SystemPhase,
    pub func: fn(&mut I) -> O,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemPhase {
    Update,
}
