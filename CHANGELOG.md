# Changelog

This file documents the most important changes for each released version.

## [v0.0.10]

## [v0.0.9]
- Added very basic 2D physics support
- Added more component lifecycle callbacks. Mainly:
    * Fixed/physics updates
    * Component started/destroyed callbacks
- Updated window API and component context API

## [v0.0.8]
- Switched out ECS for more traditional GameObject oriented design. 
    * ECS was getting too complicated from a user-API standpoint. It required too many macros to be "nice"
    * I found ECS simply too annoying to develop, to be honest
- Support for borderless/exclusive fullscreen
- Introduced multithreaded main loop
- Basic inter-object message passing
- Improved keyboard input API to include pressed/released this frame along held/not held

## [v0.0.7]
- Initial support for optional components in ECS queries
- Camera component now influences rendering through its position (as determined by its Transform component) and its perspective settings
- Engine plugins can now listen and respond to raw window and device events
- Basic keyboard input handling
- Added time management functionality. For now, this only contains the frame start time and the delta time, but it might contain more information in the future

## [v0.0.6]
- Complete refactor of core ECS system. We now use archetypes. It was a lot of work.
- Complete user-side API refactor.
- Updated Rust version
- So many changes, just look at the changelog

## [v0.0.5]
- Initial support for material parameters (and the corresponding uniforms in the rendering backends)
- Initial support for logging and granular logging configuration
- First version of Material component

## [v0.0.4]
- Fixed various compilation and pipeline errors

## [v0.0.3]
- Generic rendering backend support
- Start of the first rendering backend: OpenGL
- First setup for a camera component
- First setup for meshes
- Support for rendering unlit 2D shapes, with a static color
- Locked toolchain for reproducible builds
- Moved a lot of public functions around for a more comprehensible user-side API (although how comprehensible is it really, without documentation)
- Editor actually opens a WutEngine window now


## [v0.0.2]
- Basic mutable and optional query qupport

## [v0.0.1]
- Basic query and command support

## [v0.0.0]

- Initial version
