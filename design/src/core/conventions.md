# Conventions and names

The conventions every subsystem shares: coordinate frame, units, rotations, depth and alpha; how things are named;
the "static" marking; and how spatial queries are split between subsystems.

## Coordinate frame and units

- **Right-handed, +Y up, metres** (and kilograms, seconds): glTF's frame. With +X right and +Y up on screen, +Z points
  out of the screen, towards the viewer. Why: it is the most common frame (glTF's and most engines'), so glTF assets
  need no axis conversion; matching the main interchange format beats any converter, which breaks on rigs and
  animation.
- **One forward, +Z, for everything:** objects, characters, cameras and lights all face local +Z, so `look_at` and
  "move forward" mean the same for all of them. Why: two forwards (models +Z, cameras −Z) is the most recurring
  convention pain in right-handed engines using glTF; one forward removes it for gameplay code, and its costs land in
  the importer and in how two cameras are created, which the engine controls. Consequences:
  - a camera looking +Z has world −X on its right, so the 2D camera and the editor's default view look along −Z at the
    XY plane and at a model's front (a camera turned 180° about Y); the editor and templates create them that way;
  - glTF cameras and lights are reoriented at import (a 180° turn about their local Y, applied to their rotation keys,
    children counter-rotated), with tests from the start; meshes and rigs need nothing;
  - a character's right is −X, as in any right-handed engine using glTF assets.
- **No engine-wide convention switch.** Per-asset import overrides with one fixed engine convention are the safe kind.

## Rotations and angles

- **Quaternions everywhere:** in memory, in files, in the Rust and script APIs.
- **Euler angles in degrees for display only:** the inspector shows and edits rotations as yaw-pitch-roll (intrinsic
  Y, then X, then Z). Why: that order suits a Y-up world.
- **Other angles:** Rust follows glam (radians); the script API uses degrees or radians per API, with every angle
  parameter naming its unit (`fovDegrees`) and conversion helpers available.

## Depth

- **Reversed Z with an infinite far plane by default,** on a `depth32float` buffer: near maps to 1, clear to 0,
  compare `greater`. Views may still set a finite far plane (orthographic views, shadow cascades).
  Why: it is core WebGPU on every target, including the web, and with a float buffer it gives far better precision;
  an infinite far plane then costs almost nothing.
- **Named WGSL helpers and documentation from the start:** linear depth, "is closer", view-space position from depth, a
  full-screen triangle. Why: reversed Z only hurts custom-shader authors, and is hard to diagnose without helpers.

## Alpha

- **Straight alpha:** textures, colours, materials and shaders use straight alpha, as image files and tutorials do.
  Why: premultiplied alpha throughout is too far from what people are used to.
- **Colour dilation at import** for colour textures with alpha, on by default with an import setting to turn it off:
  transparent pixels get the colour of their nearest visible neighbours. Why: filtering and mipmaps then don't pull in
  black, which removes most dark fringes.
- **A premultiplied blend mode as an opt-in** render state on materials, for premultiplied textures and effects mixing
  glow and cover in one texture.
- **Internal passes may use premultiplied alpha** where needed and invisible to users: egui, compositing an offscreen
  layer, a transparent web canvas.

## Following from the APIs

- Matrices are column-major with column vectors (`m * v`); quaternions are (x, y, z, w). glam, WGSL and glTF agree.
- Texture coordinates start top-left; glTF and PNG upload without flipping.
- Normal maps are +Y ("OpenGL style", as glTF); an import option flips green for DirectX-style maps.
- Front faces are counter-clockwise; back-face culling is on by default for surfaces. Mirrored transforms (negative
  determinant) flip winding; the renderer handles that per instance.

## Importer requirements

Per-asset import overrides for up axis, forward axis and scale; FBX and USD axis and unit metadata read from each file
and converted (rotations or scales, never a mirror); tests on skinned meshes, cameras, lights and animation tracks
([Assets and project files](../world/assets.md)).

## Names

A rule for the whole engine on what is a name, what is a string and what is a type:

- **A `Name` for runtime identifiers authored as data:** whatever code looks up or compares while the game runs and
  that is authored in the editor or asset files: shader and material parameters, animation parameters, bones, tags,
  input actions, pass names. One engine type, not `String`.
- **Plain strings for text and paths:** text people read (entity display names, unlocalized UI text, log messages;
  localizable text has its own type, [In-game UI and text](../systems/ui_text.md)), paths, property names in files,
  profiling markers.
- **Types for what code defines:** components, configuration sections, signals ([Signals](../scripting/signals.md)).
- **Identity is never a name:** UUIDs in files, entity references at runtime.

What a `Name` is:

- **A 64-bit hash.** The same value in every process, tool, pack and network message, and a compile-time constant
  through a plain `const fn` (`const TINT: Name = Name::new("tint");`, no macro). Why: names cross process boundaries
  (the play-mode process and editor link, tool-generated data such as shader reflection and imported bone names,
  runtime-ready packs, the network); a hash is the same everywhere, while an interned index would need the text at
  every boundary. Hashes are also usable in `match` and need no shared table on the path of parallel loading.
- **A text table in every build:** every `Name` made from a string is recorded with its text, so logs and errors show
  `tint`, not a number, also in shipped games. Compile-time constants carry their text in development builds; in
  release builds a constant that matches no loaded data logs as a number.
- **Collisions are caught where they can occur:** a hash already in the table with different text is an error in
  development builds and the editor, and the build hashes every name in the project and fails on a collision.
  Why: with 64 bits collisions are very rare and trivial to fix by renaming, but must never pass silently.
- **Text in editable files, hashes only in packs and on the network.** Why: changing the hash function later then only
  means rebuilding packs, never converting saves.
- **Case-sensitive,** as Rust, JavaScript and WGSL identifiers are. Matching names that differ only in case is a
  warning for the editor, not the type's job.
- **Scripts pass strings;** the engine hashes at each call. Why: JavaScript numbers can't hold a 64-bit hash, and the
  cost is small next to the call's own crossing into Rust.
  Kept possible: a script `Name` object as a fast path (APIs typed `string` can later accept `string | Name`), added if
  profiling shows the conversion matters; generated `.d.ts` types listing known names as string literal unions.

## The "static" marking

The marking means "this object's transform never changes during play". It exists for work done before play: baked
lighting, navmesh baking, build-time mesh merging, static shadow maps, prebuilt physics trees. Runtime caches don't
need it: transform propagation, GPU scene uploads and interpolation already work from change tracking. The rule is
fixed now and built with its first consumer, so later topics share one meaning instead of each adding a flag.

- **One marking, "Static", on the entity,** meaning only "never moves". What each bake includes is a setting on that
  system's own component (whether a static mesh contributes to baked lighting, whether a light is baked, whether a
  collider counts for the navmesh). Why: a rock is marked once, not four times, and the settings can't disagree about
  whether it moves.
- **Set in the editor or scene data, read-only at runtime** (readable by scripts and Rust). It covers the transform
  only: changing a static entity's mesh, material or enabled state is allowed.
- **Off by default.** Why: everything works out of the box, including moving any object from a script; a jam project
  never has to know about it. The editor's bake windows show what is and isn't static.
- **Moving or reparenting a static entity at runtime is refused with an error:** the transform write does nothing, and
  an error is logged at the caller's line, once per call site with a count, as for any invalid call
  ([Error handling](errors.md)). The script isn't stopped. Why: baked data can then never disagree with the object, and
  the user sees why nothing moved. Origin shifting and editing in the editor aren't moves.
- **Per entity, every ancestor static:** the marking isn't inherited, but a static entity's ancestors must all be
  static (an editor error otherwise, with a "make parents static" fix); its children may move (a door under a static
  room). An editor command "mark static, with children" marks a whole level in one step. Why: a static entity can then
  only move if it or an ancestor moves, which the refusal above catches.
- **Instances:** placing a scene instance at spawn or at load isn't a move; static entities in a spawned instance are
  static from the start, and the ancestor rule follows the instance's parent chain ([Scenes](../world/scenes.md)).
- **Physics:** a collider without a rigid body is fixed whatever the marking; a rigid body or character controller on
  a static entity is a contradiction the editor flags ([Physics](../systems/physics.md)).

## Spatial queries

- **Gameplay queries are physics queries over colliders.** Raycasts, shape casts and overlaps answer from the physics
  world and find colliders ([Physics](../systems/physics.md)). Anything that should be findable gets a collider; a
  trigger covers spawn points, pickups, interaction zones and AI points of interest. Entities without a collider are
  found by type or tag ([Scenes](../world/scenes.md)) and filtered by distance with a plain loop.
  Why: it is what every engine users may know does; an engine-wide index over every entity would be updated every
  frame for queries most games never make.
- **Many problems, not one: each subsystem owns its spatial structure** and designs it with its own topic: physics its
  broad phase and query structure, the renderer its culling data in the retained GPU scene, audio and navigation their
  own. Shared code is fine (a bounding-volume type, a BVH or grid type in engine core, built when a first subsystem
  needs it). Why: subsystems want different objects, bounds, update rates and even poses (physics the simulated pose,
  rendering the interpolated one).
- **Query positions are `WorldPos` in Rust and absolute f64 in scripts** ([Game size and
  precision](../goals/game_size.md)); origin shifting, when it exists, shifts the structures through each subsystem's
  shift hook.
- Kept possible: an engine-wide spatial index or k-nearest queries, as an extra query source if a game needs them.
