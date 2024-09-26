use core::any::TypeId;
use core::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub struct ReadWriteDescriptor {
    pub type_id: TypeId,
    pub read_only: bool,
}

pub struct System<I, O> {
    pub phase: SystemPhase,
    pub read_writes: Vec<ReadWriteDescriptor>,
    pub func: for<'a> fn(&'a I) -> O,
}

impl<I, O> Debug for System<I, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("System")
            .field("phase", &self.phase)
            .finish()
    }
}

/// The different phases where a system can run
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemPhase {
    /// Run the system each frame
    Update,
}
