# Documentation

How the engine is documented for the people who make games with it, and what the manual must teach because a decision
relies on it.

## Two kinds of documentation

1. **An API reference for Rust,** generated from doc comments with rustdoc.
2. **A manual in Unity's style, written as an mdBook:** explanations, workflows and guides for game creators, next to
   the reference rather than inside it.

**Versions:** the documentation published online is versioned per engine release; the copy in the repository isn't
(it describes the current code).

## The script API reference is generated from the registration

- **One source, several outputs.** The registration that produces the generated `.d.ts` (`webuild types`,
  [Scripting](../scripting/README.md)) also carries each exposed item's description, taken from its Rust doc comments.
  The same run writes **JSDoc in the generated `.d.ts`,** so code editors show the descriptions on hover, and
  **Markdown pages** for the reference section of the manual, versioned with the online docs. The editor can show the
  same help later.
- Why: no Node and no TypeScript documentation tool in the docs build, and one source that can't drift. Cost: the
  Markdown writer is the engine's own.
- **The doc strings are compiled into the play-mode player and the editor,** never into the release player that ships
  with a game ([Building and shipping](shipping.md)).

## How the manual and the references link

The generated reference pages live inside the manual; manual pages link to them with ordinary relative links, and doc
comments link back to manual pages with a fixed prefix the generator resolves. A broken link fails the docs build.

## What the manual must teach

Decisions that rely on the documentation:

**Scripts**
- Entity scripts wait and connect through `this` (`this.wait(...)`, `this.connect(...)`), so the work ends with the
  component; a plain `await load(...)` keeps running after the entity is gone ([Script API](../scripting/api.md)).
- `finally` after a component's wait doesn't run when the component goes away; cleanup goes in the destroy callback.
- Awaiting something that never comes (a sender destroyed before it emits) hangs silently.
- No `setTimeout`; repeating timers are a loop around a wait; "wait until a condition" is a loop over `waitFrame`.
- Hot reload and handlers: a handler written as `(x) => this.onOpened(x)` picks up new method code after a reload;
  passing `this.onOpened` directly, or a long arrow body, keeps the old code until the connection is made again.
- Quitting handlers: code after an `await` never runs once quitting; `request.cancel()` on quit requested must come
  before the handler's first `await` ([Signals](../scripting/signals.md)).

**Input and camera** ([Input and camera](../systems/input_camera.md))
- Simulation in the fixed update reading actions; look and feel per frame.
- A camera that follows by code moves in the late update; mouse look is applied per frame; code following an
  interpolated entity reads its displayed pose.
- A click on an in-game monitor becomes the hit's texture coordinates times the texture size.

**UI** ([In-game UI and text](../systems/ui_text.md))
- Text-size minimums from accessibility guidelines.
- Disable the gameplay action set while a menu has UI focus.
- A HUD that stays across levels lives in a scene kept loaded.
- A widget scene puts its `Button` on its root or exposes it as a scene property, since a reference into an instance
  reaches only its root and scene properties.
- Layout runs once per frame, so code that sets a text and reads its size sees the new size a frame later.
- A pause menu's entities are marked "runs while paused", or a menu script awaiting a signal hangs.

**Audio** ([Audio](../systems/audio.md))
- A sound played with `audio.play` outlives its entity; an `AudioSource`'s sound stops with its entity.
- Calls on a finished playback do nothing; `finished` always settles, also for a sound that was never heard.
- On the web nothing is heard until the first click, tap or key press, while the game runs and keeps sound time.
- A pause menu's sounds go on a bus that doesn't pause with the game; "runs while paused" doesn't affect sound.

**Physics** ([Physics](../systems/physics.md))
- Forces and kinematic moves belong in fixed update; from `update` forces depend on the frame rate and kinematic bodies
  push in jerks.
- Queries ignore triggers by default, so a query for pickups sets `triggers: true`.
- A destroyed partner's handle in an exit event throws; a kinematic body touching a fixed or kinematic one gives no
  collision event.
- How a non-uniform scale bends a sphere or capsule.
- Turning a character controller's entity per frame loses its interpolation; turn the camera child instead.

**Rendering** ([Renderer architecture](../rendering/architecture.md))
- Light units (lux, lumens, nits) and EV100 exposure in plain words, with intensity presets.
- Tonemapper None shows texture colours exactly; PBR Neutral darkens unlit colours slightly.
- UI blends in sRGB like image editors; UI ordered before a camera is part of the scene.
- Readback from the GPU arrives at least one frame later.

**The editor** ([Editor](README.md))
- Edits made in the editor during play mode are kept; what the running game changes ends with play mode.
- Pressing Play saves unsaved documents first; selection and file operations aren't undoable.
- Until crash recovery is built, an assert or a GPU device loss in the editor loses unsaved work.

**Save games** ([Save games and storage](../world/save_storage.md))
- A save holds game data the game chooses, with its own `version` field and migration.
- On the web, call `saves.requestPersistence()`; browsers may delete a site's data (Safari after seven days without a
  visit), itch.io's iframe partitions storage, and a save should start when the game is backgrounded.
- Where saves, settings and logs live per platform.

**Packages** ([Packages](../world/packages.md))
- For package authors: never regenerate asset UUIDs; they are the package's contract.
- A package's code runs with the project's rights once the editor loads or compiles it.
- Updating replaces the package folder; git shows local changes.
