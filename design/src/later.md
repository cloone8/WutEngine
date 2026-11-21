# Later

Features that plug into the design later without changing it, designed when they are built. Listed so nothing is
forgotten; none of them has a design yet.

## Wanted, not early

- Baked lighting: light maps, a bake tool, a light-map unwrap at import (UV1 already has a place in the mesh format).

## Nice to have, not a priority

- VR and XR: nothing extra designed; views (stereo is two views) and the late-latching door keep it possible.
- Modding and user-generated content: game authors can expose their own modding API.
- Hot reload of Rust game code and dynamic linking in development builds: dropped if impractical.

## Engine core

- Frame pacing: presentation timestamps for frame-time snapping, a "low latency" option (one frame in flight), late
  latching.
- Each window rendered at its own monitor's refresh rate, decoupled from the simulation and interpolated, if judder
  on monitors with another refresh rate than the main window's ever matters (most likely for the editor).
- A step cap setting for the fixed-update loop, if the delta clamp ever isn't enough.
- An own job scheduler replacing rayon, if its overhead shows in a real game's profile, or by interest.
- An engine-wide spatial index and k-nearest queries, if gameplay needs more than physics queries.
- Quality presets over settings.
- Aligning capped frames to the observed input or display phase.
- On the web, a "warn before leaving" setting mapped to `beforeunload`.
- Memory budgets per subsystem and memory warnings on mobile.

## Rendering

- Mipmap generation and filtering (trilinear, anisotropic), volume, procedural and animated textures.
- Parallax occlusion mapping.
- Texture streaming and virtual texturing, with GPU out-of-memory degrading (evict, drop mips) instead of fatal.
- Billboards and impostors, mesh preparation at import, simplification beyond LODs.
- Point and voxel rendering, pre-rendered backgrounds.
- A per-object LOD bias, a dithered LOD cross-fade, different meshes per LOD level; eight skin influences.
- Occlusion culling (hierarchical Z), portals, terrain, water.
- Tile-based GPU bandwidth work, specular occlusion, parallax-corrected environment probes, caustics.
- Depth prepass, stencil as render state, GPU picking.
- Dynamic resolution and temporal upsampling.
- Render scale: the 3D scene at a fraction of a high-DPI window's pixels, upscaled, UI at full sharpness.
- Order-independent transparency, alpha to coverage, FXAA and SMAA, specular antialiasing.
- Area lights, cookies and IES profiles, irradiance probes and volumes, baked AO.
- Shadow filtering (PCF, PCSS, VSM), contact and volumetric shadows.
- Planar reflections, blended environment probes.
- Subsurface scattering and skin, cloth, clear coat and anisotropy, thin film, hair and fur, refraction.
- A physically based sky, clouds, height fog, light shafts.
- Depth of field, motion blur, lens flare, blur building blocks, temporal reprojection beyond TAA.
- Hatching, painterly, line and wireframe rendering, edge detection.
- A deferred path, visibility buffer, deferred texturing, object- and texture-space shading.
- Screen-space reflections, global illumination, volumetrics, decals, 2D lights.
- Colour grading as a 3D LUT before tonemapping; AgX and ACES tonemappers.
- HDR display output.
- A low-end option without the HDR target.
- Binding arrays and bindless as a native accelerator (a `weshc` variant); GPU-driven rendering.
- Shaders compiled to each backend's own format at build time and loaded as passthrough shaders, skipping naga's
  translation at runtime: a possible stutter win in exchange for a little unsafety (the build's validation is then
  the only check). The driver still compiles pipelines on the player's machine; the pipeline cache helps there.
- Render layers per view (a first-person weapon camera, a minimap).
- Light intensity presets in the inspector (candle, bulb, overcast, sun).
- Render bundles to record one heavy pass in parallel chunks.
- Custom node assets holding same-named type variants, picked by the connected type.
- GPU jobs for scripts, with a promise-based readback.
- Generated typed Rust bindings for compute shaders.

## Animation, VFX and 2D

- Skeletal animation, blending and IK (earlier if the first game needs animated characters).
- Particles and VFX, including graphs that generate compute work.
- Sprite atlases packed at import (sprite frames are atlas regions); tilemaps.
- 2D physics: `RigidBody2D`, `Collider2D` and a separate 2D world.

## Navigation and AI

- Pathfinding with navmesh lookup; perception (nearby targets, line of sight through physics raycasts).
- Decision making, group and crowd behaviour.
- Entity and simulation LOD for far-away game state.

## Physics

- Joints, ragdolls, destructibles, camera collision, cloth, soft bodies, buoyancy, fluids, vehicles.
- Prebuilt static query trees in build packs, merged when a scene loads.
- Per-collider layer overrides and a per-pair ignore.
- Per-material combine modes; "stay" events.
- Moving platforms, crouching and networked prediction for the character controller.
- A surface type per triangle of a mesh collider; per-scene gravity.
- Collider and trigger gizmos in the editor; a physics debug view in the overlay (colliders, triggers, contacts).
- An in-house physics solver (unlikely: rapier is assumed permanent).

## Audio

- A dialogue system, a music system, interactive music.
- Sounds driven by parameters; tail clips when a loop stops.
- Bus effects, ducking and snapshots.
- HRTF, Doppler, reverb regions, obstruction through physics rays, surround output.
- Several listeners.
- The audio clock and scheduled starts; pitch following the time scale per bus.
- Opus, re-encoding at import, the browser's decoder on the web, streaming from disk.
- Hot reload of sounds.
- A `blocked`/`unblocked` state on the audio service for "click for sound" hints.
- Third-party middleware integrations that bring their own components.

## Input and windowing

- Interactions (hold, tap, multi-tap, hold-or-toggle); chords, combos and gestures; motion sensors and camera input.
- A "press a button to join" helper for local players.
- Response curves on analog bindings; controller lights.
- Events on actions, as signals.
- Normalized screen conversions on the camera (`viewport_to_ray`, 0 to 1 in the camera's view).
- Custom cursor images.
- Exclusive fullscreen, if a platform needs it.
- Several canvases on one web page.
- Window position; a pixel-exact window size; opting a window out of high-DPI.
- The application and window icon: mostly a build setting (embedded in the executable or bundle), plus the window
  icon at runtime where a platform shows one (Windows, X11).
- IME on the web; an on-screen keyboard for gamepad-only text entry.
- Mobile specifics: safe areas, orientation, screen density.

## In-game UI

- Widgets beyond label, image and button: toggle, slider, text field, scroll view, dropdown.
- Styles and themes shared across widgets.
- UI animation and transitions (on real time in pause menus).
- UI markup and stylesheets; data binding; UI screens placed directly in 3D.
- A pointer helper for UI on a fullscreen upscaled render texture.
- A separate text size for reflowing layouts; per-local-player UI focus.
- Rich text, colour emoji, text along paths.
- Pseudo-localization and translator exchange formats (PO, XLIFF, CSV).
- Per-locale assets beyond fonts; editor localization.
- Screen-reader adapters and stored accessibility overrides; screen readers on the web.
- Text-to-speech, subtitles, colour-blind filters.
- Connections wired in the editor (signal to entity and method, inside one scene).

## Scripting

- Hot reload level 2: swapping code while playing, keeping state.
- Debugger support through hooks contributed upstream to boa.
- Script jobs on worker VMs.
- A wait for the next fixed step, cancelling a single wait, "wait until a condition" as an API.
- Rust systems reading signal emits as data; opt-in deferred connections.
- A `changed` signal on configuration sections.
- A script `Name` object.
- Visual scripting.

## Gameplay and content

- State machines.
- A follow camera component reading a target's displayed pose (offset, damping).
- Orbit cameras, spring arms, shake as a layer on the view, camera blending; late latching of the camera.
- Splines and paths, with an editor gizmo for control points.
- Object pooling, as documentation only.
- A sequencer or timeline for in-game cinematics.
- Video playback for cutscenes and video textures.
- Level geometry authoring (blockout, brushes), terrain tools.
- DCC round-trip and live links; data-processing graphs for assets.

## Assets, scenes and saves

- Loading and freeing a cycle of hard asset references as one group.
- Scene variants (a scene based on another, overriding anything) and per-instance editable children.
- Patches and DLC through manifests that override earlier ones.
- Automatic pack grouping by scene on the web.
- Prebuilt players for script-only projects; a "development build" player.
- macOS signing and notarization.
- An engine helper saving a scene's changed entities as overrides.
- Save compression, encryption and signing.
- Exporting and importing a save as a file download on the web.
- Steamworks and platform save back ends.

## Editor

- Picking, selection highlighting, transform gizmos for any component.
- Editor layers, snapping, camera navigation.
- Content validation warnings; a semantic 3-way merge driver for scene files.
- Crash recovery through the change journal (required before jam-ready).
- The editor's asset database (references, unused assets, reimport), a cache rebuilt by scanning the project.
- A start-up recovery mode; a navigation history.
- Copying play-mode values back; a live-edit switch; path matching on paste; a brush-stroke diff helper.
- Embedding the play-mode window in the editor window.
- The editor link over WebSocket, for web and mobile games.
- Native popups, owner and modal windows.
- Collaboration beyond version control.

## Debug tools

- A "pending waits" view in the script debugger.
- Crash reports with a monitor process (dumps, upload).
- A strict mode making logged errors fatal, for automated runs.
- Developer menus, free camera, cheats, screenshots and video capture.
- CPU memory stats and leak detection, on the engine's memory counters.
- Replay and input journaling; games writing the per-step input record themselves.
- GPU-side printf and debug drawing.

## Ecosystem

- An API versioning and deprecation policy before 1.0.
- Packages from a git URL; declared, fetched packages and a registry; npm packages.
- Package version ranges and conflict resolution; sandboxing package code; a package config layer.
- Switching unused engine features off for smaller builds.
- The engine's templates shipped as packages.
- Template scripts: a "game events" script on a global entity, a "save system" script.
- Multiplayer details, anti-cheat and data integrity.
- Testing, mobile, onboarding (a guide and a template project).
