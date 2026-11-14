# Object model

How game objects exist in WutEngine: one ECS world underneath, seen as objects by the editor and scripts, with a
transform hierarchy, stable identity and fixed rules for creation and destruction.

## One world, two views

- **Underneath: an ECS.** Entities are IDs with a generation, so they are weak references: a destroyed target is
  detectable. Components are plain data; Rust systems run over declared queries
  ([Time and the frame](../core/frame.md)).
  Engine subsystems (physics, rendering, audio) keep their own data-oriented storage behind proxy components.
- **The editor and scripts see objects.** In the editor an entity looks like a GameObject: a name, a transform, a
  parent and children, a list of components. Script components are classes with exported properties
  ([Reflection](reflection.md)) and per-object callbacks (`update`, `onCollision`), run sequentially on the script
  thread with immediate semantics, reaching other entities through entity references
  ([Script API](../scripting/api.md)).
- Why: pure ECS for everyone is unfamiliar to non-programmers ("where does my script go?"), while object-only engines
  keep gameplay on the main thread and later add ECS-like layers for performance, ending up with two worlds to sync.
  Here it is one world, so there's nothing to sync.
- **No engine-provided gameplay framework** (game modes, player controllers): game flow is left to each game. Global
  entities without a transform (below) cover "GameManager"-type objects.

## The ECS crate: hecs, behind the engine's own API

The ECS underneath is the `hecs` crate, wrapped in WutEngine's own API.

Why hecs:

- **Deterministic query order** (archetypes in creation order), the same natively and on wasm
  ([parallel equals sequential](../core/frame.md#parallel-equals-sequential)).
- **Safe parallel queries from `&World`,** so the engine's own scheduler needs no `unsafe`.
- **Small, with no scheduler, resources or events of its own.**

What the engine builds on top:

- per-system access lists from its own system declarations (the [Rust scheduling](../core/frame.md#rust-scheduling)
  model);
- changed-component tracking;
- type-erased component access through registered functions;
- the transform hierarchy.

## Entities and components

- **One component per type per entity.** For scripts: one instance of each script class per entity; different script
  classes on one entity are fine; multiples go on child entities. Why: several of one type has complex implications
  for little gain.
- **Every scene entity has a transform and a place in the hierarchy.** Why: the editor, scene instances and origin
  shifting need it.
- **Entities without a transform** are allowed, for "GameManager"-type objects. The editor shows them in a separate
  list next to the hierarchy, with components and scripts but no transform, parent or children. Two lifetimes:
  - **per scene:** saved in and loaded with a scene, owned by the root of their scene or scene instance
    ([Scenes](scenes.md));
  - **global:** a project-level list, created when the game starts, before the start scene is loaded, and surviving
    scene changes.
- **Identity:** a UUID per entity in scene files; the runtime ID is separate and chosen for speed. While loading, a
  UUID-to-runtime-ID table resolves references ([Assets and project files](assets.md)). Overrides on scene instances
  are keyed by stable IDs (a scene property's ID), never by array index ([Scenes](scenes.md)).

## Transform propagation

Each entity has a local transform (relative to its parent) and a world transform.

- **Scripts: computed when read.** A world-transform read computes what's needed and caches it, so scripts always see
  correct world transforms. Why: scripts run sequentially while no Rust systems run, so this is safe and cheaper than
  updating all descendants on every write.
- **Rust systems and the engine: one parallel pass over dirty subtrees** at each phase boundary, and at least before
  physics and before rendering. Within a phase, Rust systems see world transforms as of the start of the phase.
  Why: if every transform write updated all descendants, transform-writing and world-reading systems could never run
  in parallel.

## Creation and destruction

- **Scripts: immediate,** since they run one at a time.
- **Rust systems: queued as commands, applied right after the group of systems that issued them,** so the next group
  already sees the change. Why: applying them only at the end of the frame causes off-by-one-frame bugs.
