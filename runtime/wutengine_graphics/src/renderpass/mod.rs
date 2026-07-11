//! Custom and built-in graphics rendering passes

use core::any::Any;

/// A render pass implementation
pub trait RenderPass<T, D>: Send + Sync + Any
where
    D: ?Sized,
{
    /// The name of the pass
    fn name() -> &'static str
    where
        Self: Sized;

    /// The order of the pass, relative to other passes. Higher is later.
    /// Most built-in passes have an `ORDER` const that you can base the return value
    /// of your custom pass on
    fn order() -> u64
    where
        Self: Sized;

    /// Construct a default version of this pass. Called once per camera
    fn construct() -> alloc::boxed::Box<dyn RenderPass<T, D>>
    where
        Self: Sized;

    /// Run the pass for the given target. Commands should be placed in `cmd`
    fn execute(&mut self, cmd: &mut wgpu::CommandEncoder, target: &T, drawable: &D);
}
