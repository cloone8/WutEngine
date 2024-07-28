use super::{ShaderSet, ShaderSetId};

pub trait ShaderResolver: 'static {
    fn find_set(&self, id: &ShaderSetId) -> Option<ShaderSet>;
}
