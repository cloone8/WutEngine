# Game size and precision

How big the games made with WutEngine can be, what that means for world positions, and what is kept possible for
open worlds and large servers.

## Scale

- **Games of any size in principle, built for what jam-to-indie games need**, with simple default templates and
  settings. Why: that is the audience; larger games get the advanced APIs ([Principles](principles.md)).
- **Only world size shapes the design now.** "Big" has several dimensions: world size (open worlds) shapes APIs, file
  formats and the editor in ways hard to change later; object count (culling, batching, parallelism) is mostly
  internal; content volume (import and build caches, packaging) is tooling; team size is partly file layout, which
  per-asset files and stable UUIDs already help. The latter three are added when needed.

## Positions

- **f32 positions with camera-relative rendering.** Documented limits: about 10 km for rendering and about 1–2 km for
  physics-heavy worlds. Why: f64 on the CPU everywhere would cost every position-handling system (measured 1.5–1.9×
  slower for transform propagation and physics) for worlds most games don't need, and the GPU stays f32 anyway (WGSL
  has no portable `f64`).
- **The position API is split.** Storage and the Rust `Transform` are f32, with local and world positions as in Unity;
  world is relative to one global origin, which stays zero until origin shifting exists. Scripts and the editor see
  absolute f64 positions (origin plus world; JavaScript numbers are f64). Rust gets a separate function returning an
  absolute `DVec3`. Why: if positions changed under scripts when the origin shifts, every script would have to handle
  the shift, a trap for non-programmers.

## Precision build switch

Large dedicated-server worlds ([Multiplayer](multiplayer.md)) can't shift their origin for players far apart, so with
f32 a physics-heavy server world stops at about 1–2 km. World positions are therefore a wrapper type, `WorldPos`,
whose precision is chosen at compile time: f32 by default, f64 in builds with the `f64_positions` feature.

- **What switches:** world positions only: `Transform`'s position, stored positions (the dedicated world-position
  type below is `WorldPos`), physics bodies (`rapier3d` or `rapier3d-f64` by the same feature), spatial queries, audio
  sources, navigation. Rotations, scales, directions, velocities and local offsets stay f32.
- **What doesn't:** rendering (the retained GPU scene stores camera-relative f32; in an f64 build the subtraction
  happens in f64 first), scripts (already f64), scene files (JSON numbers, so the same project loads in both builds).
- **A wrapper, not a type alias.** Why: with an alias, the type is `f32` in the default build, so wrongly mixed code
  compiles and only breaks in the f64 build; the wrapper makes mixing a compile error in both builds.
- **Costs accepted:** explicit position operations (`pos + offset`, `pos.distance(other)`); checking the f64 build
  occasionally with clippy; slower position math, only in builds that turn it on.

## Open worlds: kept possible, nothing built

- **Streaming is loading and unloading scenes additively**, with the game splitting its world into scenes
  ([Scenes](../world/scenes.md)). Not built: automatic world partitioning, streaming cells, a partial-world editor, a
  "maybe not loaded" script API.
- **Origin shifting is kept possible.** When the camera or player passes a threshold, an offset is subtracted from
  every world-space position and added to the global origin; velocities are unchanged. Root transforms, physics
  bodies, world-space particles, trails and decals, audio sources, navigation and previous-frame view data must shift.
- **Groundwork built now, because it is cheap now:**
  - scripts see absolute positions, with the engine converting at the boundary;
  - a dedicated world-position type for positions stored in components (patrol points, last-seen positions), so
    reflection can find and shift them;
  - save files and networking always use absolute positions;
  - one "shift" hook per subsystem that caches world positions.

  Why: game code keeping world positions across frames would silently point to the wrong place after a shift, and
  retrofitting this later is very expensive.
- **A retained GPU scene keeps instance positions camera-relative** (or otherwise precise enough).

## Loading

- **Async asset loading and several scenes loaded at once, from day one.** Why: the web requires async loading;
  additive scenes are the basis for streaming and give teams fewer merge conflicts. Loading a scene loads every asset
  it references, so small games never see loading states, while runtime asset loads are async so larger games can keep
  scenes small ([Scenes](../world/scenes.md), [Assets and project files](../world/assets.md)).
