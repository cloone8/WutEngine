# Scenes

What a scene is, how scenes are loaded, switched and instanced at runtime, what an instance may change, and how
entities are found.

## One asset type: the scene

- **One word, "scene."** What other engines call a prefab is an **instance** of a scene. The engine and editor don't
  use the word "prefab" (documentation may mention it for Unity users). Why: two words for one thing would suggest a
  difference that doesn't exist.
- **One file format and one asset type** for an entity hierarchy, used in two ways: **loaded** (its own unit of
  lifetime) or **instanced** (copied into a scene that already exists, at a transform, owned by that scene). Nested
  instances and "a level made of pieces" are the same mechanism. Why: with additive scenes, loading a scene and
  spawning an instance both add a set of entities to the world; two formats would duplicate the loader, overrides,
  references and editor tools.
- **Every scene has exactly one root entity,** the main scene included. The root is the unit: on an instance it holds
  the placement and shows the scene properties, and a loaded and an instanced scene have the same structure. The editor
  shows the root *as* the scene (the scene's name at the top of the hierarchy; selecting it shows the root's
  components), so it isn't an extra object to a non-programmer. Its transform is identity by default.
  Why: several roots would need an invented parent for an instance.

## The scene API

- **`load(scene, transform)`** adds a scene and returns a `Task` ([Jobs and async](../core/jobs.md)) that completes
  once the scene is in the world: its hard references loaded, entering at frame start. The optional transform places
  its root. The same scene may be loaded more than once; each load is its own **loaded scene** (the value the task
  returns), so copies can be told apart and unloaded separately.
- **`unload(loaded_scene)`** removes its entities at the end of the frame and releases what they held.
- **`switch(scene)`** is the common case: every loaded scene is unloaded and the new one loaded. Global entities are
  the only survivors (no "don't destroy on load" flag). Shared assets are kept through the switch, and the engine
  forces a script garbage collection there, and only there ([Assets and project files](assets.md)). To scripts it
  looks synchronous: the call returns, and the new scene's scripts start once it's in; nothing in the old scene runs
  meanwhile.
- **The main scene** is the one the game started with or last switched to, or the one chosen with `setMain`; when it is
  unloaded, the most recently loaded remaining scene becomes main. Spawns without a parent go there. The build setting
  naming the first scene is the **start scene** ([Building and shipping](../editor/shipping.md)). Gravity is a project
  setting, not a scene's ([Physics](../systems/physics.md)).
- **Nothing with a transform survives `switch`.** The documented pattern for a player character that persists across
  levels is to keep a "Player" scene loaded and change levels with `load` and `unload` instead. A seamless transition
  is the same pattern: load the next scene, then unload the old one. Why no built-in seamless switch: it doubles the
  memory peak.

## Instancing at runtime

- **`spawn(scene, transform, parent)`** with a loaded `Asset<Scene>` is immediate for scripts and a command for Rust
  systems; from an `AssetRef` the game loads first. The new entities belong to the parent's scene; without a parent
  they go under the main scene's root. Moving an entity into another scene is refused.
- **Each spawned entity gets a fresh runtime ID,** and entity references inside the instance point at the new copies.
- **Runtime instances don't follow later edits of their scene.** A hot-reloaded scene applies to later loads and
  spawns; entities already in the world keep theirs.
- **No pooling in the engine:** it is game code where a game measures a need.

## Scenes are units: scene properties

A complex scene used inside another must not look complex while editing the outer one. It offers **scene
properties**, the safe way to let the outer scene change it.

- **A placed instance can change only** its root's fields (its transform among them) and the scene's **scene
  properties**: fields on any entity inside, or entities as reference targets (a spawn point), that the scene's author
  marks and names. They show on the instance as the scene's own properties (a Torch scene with `Light.color` and
  `Flame.size`). Each scene property carries a stable ID, which keys the instance's overrides, so renaming it, or
  anything inside the scene, doesn't break instances.
- **Nothing else inside is overridable;** to change something else, make it a scene property or unpack the instance
  into plain entities (no link to the scene). Cost accepted: one torch with a different flame needs a scene property or
  an unpacked instance.
- **An instance may gain things** that don't depend on the scene's internals: child entities under it, components on
  its root.
- **Nesting to any depth;** each level sees only the scene properties of the level below, and a scene may pass an inner
  instance's property on as its own.
- **Lists:** an override replaces the whole list, never single elements by position.
- **Stale overrides** (a scene property or field removed) are kept in the file and shown in the editor, not dropped;
  serde's `alias` carries renamed fields over.
- **Transform-less entities belong to a root:** they get an **owner**, the root of the scene or instance they come
  from, used for lifetime and the editor only. Each instance gets its own copies; destroying the root destroys them. In
  the containing scene's editor they stay inside the unit, changed only through scene properties. Something game-wide
  is a global entity ([Object model](object_model.md)).
- **Identity inside instances:** two instances of one scene, or a scene loaded twice, share their inner UUIDs. In a
  file an entity inside an instance is identified by its chain of UUIDs (the instances' UUIDs, then its own); at
  runtime a lookup by UUID is scoped to one loaded scene. This applies to reference resolution, the editor link and save
  games.
- **Static entities in instances:** placing an instance isn't a move ([Conventions and names](../core/conventions.md)).
- Apply and revert, and how the editor shows scene properties and overrides, belong to the
  [Editor](../editor/README.md).
- Kept possible: variants (a scene based on another that overrides anything); per-instance "editable children".

## References between scenes

- **Saved references stay inside one scene file** (including the instances in it); the editor prevents a saved
  reference into another scene. A reference into an instance reaches only its scene properties. Why: loading order
  between scenes then never matters, and every reference lookup at load succeeds.
- **At runtime, references may point anywhere;** a reference into an unloaded scene reads as gone, like a destroyed
  entity.
- **Scenes find each other's entities at runtime** by tag or type, through global entities, or connect with signals
  ([Signals](../scripting/signals.md)). Cost accepted: wiring a button in one scene to a door in another is done at
  runtime, not in the editor.

## Finding entities

- **By type:** Rust uses ECS queries; scripts get `findAll(Type)` and `find(Type)` (a Rust component or a script class)
  over an index kept by the engine. `find` is for things there's one of, with a warning in development builds when
  more than one matches.
- **By tag:** tags are `Name`s declared in the project ([Conventions and names](../core/conventions.md)); an
  undeclared tag is logged in development builds. Several per entity, indexed, so `findAllWithTag` is fast.
- **By name, only `findChild(name)`** under one entity (the Muzzle under my Gun). **No global find by name.** Why:
  names are labels, not identity, and may repeat; tags do that job.
- **By UUID:** for tools, the editor link and save games, scoped to a loaded scene; not a gameplay API.
- Searches see every entity in the world, those inside instances included: the unit principle is about editing, not
  about what exists at runtime.
- Kept possible: hierarchical tags.

## No special object types

- **Markers and spawn points** are entities with a transform and, if wanted, a tag or a script.
- **Triggers** are colliders marked as triggers ([Physics](../systems/physics.md)).
- **Splines and paths** come later, as a component with an editor gizmo.
- The engine adds a type only where data or the editor needs something an entity with components can't give. Editor
  shortcuts ("Create → Spawn point") are editor conveniences.

## Kept possible

Seamless switches as a built-in; moving an entity between scenes; derived UUIDs for entities spawned at runtime
(network identity, [Multiplayer](../goals/multiplayer.md)); automatic world cells.
