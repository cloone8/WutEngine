# Physics

The physics API: bodies and colliders, who writes the transform, queries, collision layers, physics materials,
collision and trigger events, the character controller, and the physics world. rapier is the implementation, behind
the engine's own API.

## Bodies and colliders as components

- **`Collider`** holds a shape (box, sphere, capsule, convex hull, or triangle mesh from a mesh asset), a physics
  material, a collision layer and a **trigger** switch. **`RigidBody`** holds the motion type (**dynamic** or
  **kinematic**), mass in kg (computed from the colliders' volume with one default density unless set), damping,
  gravity scale and "fast-moving". The motion type can change at runtime (picking up a crate makes it kinematic).
- **A triangle-mesh collider on a dynamic body** gets an editor warning and is simulated as its convex hull.
- **A collider without a rigid body is fixed.** Moving one teleports it without pushing anything and doesn't wake what
  rests on it; to push, use a kinematic body. In the editor and the documentation it is "a collider without a body",
  never "static": "static" is the marking for entities that never move
  ([Conventions and names](../core/conventions.md)), and a collider on an entity marked static can't move at all.
  **A `RigidBody` or `CharacterController` on a static entity is flagged by the editor,** since every move it would
  make is refused.
- **Compound bodies:** colliders on child entities belong to the nearest ancestor with a `RigidBody` or
  `CharacterController`, so a chair is one body with four leg colliders. A child with its own `RigidBody` is a
  separate body.
- **Hierarchy changes:** which body a collider belongs to is re-evaluated when the hierarchy changes; a dynamic body
  ignores its parent's motion; a kinematic child of a moving parent moves as a kinematic move.
- **Scale:** colliders follow their entity's world scale.

## Who writes the transform

One rule per motion type:
- **Dynamic:** physics writes the entity's transform after each step. Code that writes it **teleports** the body
  (velocity kept, applied at the next step; queries see it at once; interpolation snaps). Velocity is changed through
  the body: `body.velocity`, `body.angularVelocity`, `body.addForce(f)` (accumulated over the next step, then cleared
  by the engine), `body.addImpulse(i)` (an immediate velocity change). Forces belong in fixed update; from `update`
  they depend on the frame rate.
- **Kinematic:** game code moves it by writing its transform, best in fixed update; the step derives the velocity from
  the move, so it pushes dynamic bodies and carries what stands on it. Moved in `update` it still works, but pushes in
  jerks.
- **`teleport(pose)`** on bodies and the controller moves without deriving a velocity and snaps interpolation. Why:
  writing a kinematic transform derives a velocity, so a respawn 100 m away would fling what it touches.
- **Sleeping is automatic;** any change to a body, or a body touching it moving, wakes it.
- **Pause:** nothing steps and no callbacks run; queries, and `move` from "runs while paused" code, still work.
- **Origin shifting** moves every body without deriving velocity, waking or firing events, and shifts the previous
  poses used for interpolation too ([Game size and precision](../goals/game_size.md)).

## Queries

Gameplay queries (raycasts, shape casts, overlaps) are physics queries over colliders; physics owns its broad phase and
query structure.

- **Functions on `physics`:** `raycast(origin, direction, maxDistance, options)` returns the closest hit or nothing;
  `raycastAll(...)` returns every hit; `shapeCast(shape, pose, direction, ...)`; `overlap(shape, pose, options)`.
  Options: which collision layers (all by default), whether triggers count (**off by default**; a query for pickups
  sets `triggers: true`), entities to ignore. **Ignoring an entity ignores its whole body or controller,** since a
  first-person interaction ray starts inside the player's own capsule.
- **A hit** gives the entity, the collider, the point, the normal, the distance and the surface type.
- **Scripts get small result arrays;** Rust also gets a variant that fills a caller's `Vec`.
- **A collider that should only be found, not collide, is a trigger.** One concept for trigger events and for being
  found by queries.
- **What a query sees is the same rule as world transforms** ([Object model](../world/object_model.md)):
  - **scripts see the current world:** before a script query, the engine brings colliders moved, created, destroyed,
    enabled, disabled, reshaped or re-parented since the last query into the query structure, so a script that moves
    or spawns something and then raycasts finds it. Why: a trap for non-programmers otherwise, against scripts'
    immediate semantics; the cost is paid once per changed collider (a fraction of a microsecond), not per query;
  - **Rust systems see moves as of the start of the phase,** so querying systems run in parallel; created and
    destroyed colliders enter or leave when the commands are applied, after each group of systems;
  - a query never returns a destroyed entity.
- **Queries use the simulated pose,** never the interpolated one, and **return exact results:** they test the actual
  shapes and never return broad-phase candidates.
- **Result order is deterministic:** casts sort by distance; overlaps, and ties, are ordered by entity ID. Why: the
  order then never depends on thread count or the structure's shape ([Jobs and async](../core/jobs.md)).
- **No batched or async query API.** Why: Rust systems that query already run in parallel, and scripts run on one
  thread. It can be added later without changing the other queries.

## Collision layers with a project matrix

- **Up to 32 collision layers, named in the project settings** (`Default`, `Player`, `Enemies`, `Pickups`, ...).
  **Each collider is on one layer; a matrix in the project settings says which layers collide.** Queries take a set of
  layers. Why: one checkbox grid instead of two bit masks per object, for creators who aren't programmers.
- **Identity:** a layer is its slot (0–31); files store the slot, the name is a label, so renaming is free. Code looks
  layers up by name (`physics.layer("Enemies")`), unknown names logged in development builds.
- **Collision layers aren't tags:** tags find entities and are unlimited; collision layers decide what collides. The
  term is "collision layer" everywhere, since sprites have sorting layers.
- **Kept possible:** per-collider layer overrides and a per-pair ignore for two specific colliders (a grenade and its
  thrower), as optional fields.

## Physics materials with a surface type

- **A physics material asset:** friction and bounciness. A collider without one uses the project's default.
  **Combining two materials uses one fixed rule:** friction by geometric mean (ice at 0 is slippery against anything),
  bounciness by maximum. Kept possible: per-material combine modes.
- **A surface type** (`Wood`, `Metal`, `Grass`), declared in the project and set on the physics material, is reported
  in hits and contacts, so footsteps and impact effects pick sounds and particles by surface type. The default
  material's is `Default`. **Identity as audio buses:** a UUID with a readable `Name`; physics materials store the
  UUID, so renaming never breaks them; code compares names (`hit.surfaceType == "Wood"`), unknown names logged in
  development builds. Why on the material: one place, the object already chosen for sliding and bouncing.
- **Kept possible:** a surface type per triangle of a mesh collider (terrain, a level mesh of grass and rock).

## Collision and trigger events

- **Four callbacks:** `onCollisionEnter(collision)`, `onCollisionExit(collision)`, `onTriggerEnter(other)`,
  `onTriggerExit(other)`, run in the fixed-update loop after the physics step ([Time and the frame](../core/frame.md)).
  **No "stay":** a script that needs "while inside" keeps a flag between enter and exit. Kept possible: "stay".
- **Who gets them:** the scripts on the entity of each collider involved, and on its body's entity if that's a parent.
- **Per body, not per collider pair:** enter when the first collider of a body starts touching or overlapping, exit
  when the last one stops. Why: a player with two colliders would otherwise enter a trigger twice. Trigger events
  carry both the own collider and the other, as a collision does.
- **A collision** carries the other entity and collider, the own collider, contact points, normal, impulse, relative
  velocity and both surface types. **It is a copy that stays valid after the callback.**
- **Buffered, delivered after the step:** callbacks may destroy, move and spawn freely; changes apply as usual, and
  physics sees them at the next step. A destroyed, disabled or unloaded collider gives its partners an exit, so trigger
  counts stay right; its `other` is then gone and its handle throws. No callback runs on an entity destroyed earlier in
  the same delivery.
- **Order is deterministic:** sorted by entity ID.
- **Events work with no opt-in:** a trigger reports **bodies** (dynamic, kinematic, the character controller) on a
  layer it collides with, not colliders without a body (a scare trigger in a room doesn't report the floor and walls
  at load). Contacts are reported without a per-body switch, except between a kinematic body and a fixed collider or
  another kinematic body, which aren't computed (a kinematic door closing on a wall gives no collision event;
  documented).
- **Rust reads the same events as data,** a list per fixed step ([Signals](../scripting/signals.md)); Rust systems
  read a step's events in the next fixed update.

## A kinematic character controller

- **A `CharacterController` component** (a capsule) with `move(displacement)`: it moves and slides along what it hits,
  with **step height, maximum slope and snapping to the ground**, and reports `grounded` with the ground hit (and its
  surface type) and what it hit. Why kinematic: game-style controls feel wrong on a dynamic body.
- **It is a kinematic body with its own capsule and a collision layer:** it fires trigger events and queries find it;
  its hits are reported in `move`'s result, not by `onCollisionEnter`; colliders on its child entities join it and
  `move` ignores them; a `RigidBody` on the same entity is an editor error.
- **Gravity and jumping are game code:** the controller moves exactly by what it's given.
- **Pushing dynamic bodies:** the character's mass in kg, zero meaning no pushing.
- **A ready first-person player:** a drop-in player scene among the template project's content, with a camera, the
  audio listener, a `CharacterController` and a `PlayerMovement` script, to drop into a level and play. The script
  walks, sprints, looks, jumps and falls using the template's input actions, with its numbers as inspector fields
  (walk speed, sprint speed, jump height in metres, gravity). **`move` and jumping run in fixed update; looking turns
  only the camera child, per frame,** since turning the controller's entity per frame would lose its interpolation. It
  is plain TypeScript in the project. Why: non-programmers start faster; a drop-in scene rather than *the* template's
  player, so 2D and third-person teams aren't handed a first-person one.
- **Kept possible:** moving platforms, crouching by resizing, networked prediction.

## Continuous collision and joints

- **"Fast-moving" on `RigidBody`** turns on continuous collision against moving bodies.
- **Joints are kept possible** (hinges, springs, ragdolls): the first game needs none, and rapier has them.

## The physics world

- **One 3D physics world holds the colliders of every loaded scene,** so additive scenes collide with each other.
  Play mode is its own process, so the editor needs no second world for it.
- **The editor keeps a physics world for queries only, never stepped,** so edit-mode tools (drop to ground, placement
  gizmos) and edit-mode scripts can raycast.
- **Gravity is a project setting** (−9.81 m/s² along −Y) that code can change at runtime; no scene owns it. Kept
  possible: per-scene gravity.
- **2D physics is a separate set of components** (`RigidBody2D`, `Collider2D`, a 2D world on rapier2d), built when a 2D
  game needs it; the 2D and 3D worlds never interact.

## rapier, behind the engine's API

- **rapier is the physics engine, assumed permanent,** like harfrust for text.
- **No rapier type appears in the user API:** components, queries, events and materials are the engine's. Why:
  scripts, the editor and the f32/f64 build switch need the engine's own types anyway, and a rapier upgrade or a change
  of library then never changes games.
- **`rapier3d`, and `rapier3d-f64` in large-world builds** ([Game size and precision](../goals/game_size.md)).
- **Single-threaded first.** Before rapier's parallel mode is turned on, the "same result whatever the thread count"
  rule is checked with rapier's own thread-count determinism test ([Jobs and async](../core/jobs.md)).
