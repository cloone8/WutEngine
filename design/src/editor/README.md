# Editor

How the editor runs games, draws scenes and is extended, and its data model: edits, undo, saving, crash recovery, copy
and paste, and live editing in play mode. The editor is a WutEngine program, built on the engine runtime with egui
([Principles and constraints](../goals/principles.md)).

## Play mode runs in a separate process

**Pressing Play starts the project's play-mode player as its own process.**
Why: one game world per process, so the engine's global state stays global; a crash in game code doesn't take the
editor down, so no unsaved work is lost; and play mode runs the real player, built with the project's code.

- **The editor link** is an opt-in runtime feature compiled only into editor-launched players (the play-mode player,
  [Building and shipping](shipping.md)). It is desktop only, so web rules don't apply to it. It identifies entities by
  their stable UUIDs (or a per-session mapping) and carries component data as reflection data
  ([Reflection](../world/reflection.md)). Kept possible: the same protocol over WebSocket, to reach games on the web
  or mobile.
- **The play-mode window starts as a separate game window.** Kept possible: embedding it in the editor window with
  OS-level, platform-specific code; cross-process texture sharing isn't used, since wgpu doesn't support it.
- **When play mode crashes or hangs,** the editor shows the message and the log tail, reports exits that skip the
  panic hook as "ended unexpectedly", and offers to stop a game that stopped responding ([Error
  handling](../core/errors.md)).

## Scene view

**The editor process holds the edited scenes as its one game world (not simulated) and draws them with the engine's
renderer.** Project Rust code and edit-mode TypeScript run there for gizmos and edit-mode behaviour. Accepted cost:
the scene view isn't literally the shipped player.

## Editor extensions

Shaped by what users should be able to do: inspectors, gizmos, edit-mode behaviour, custom windows, importers and
build hooks.

- **TypeScript is the main editor extension language,** run in the editor's own script VM: declarative inspector
  attributes (ranges, tooltips, buttons calling functions), windows and tools, gizmos, edit-mode behaviour, importers,
  build hooks. TypeScript editor changes apply without restarting the editor.
- **Rust for complex extensions,** with full egui access, through a **project-built editor:** the editor is a library,
  and a project with Rust code compiles its own editor together with its code and extensions. A Rust change means
  recompiling and restarting the editor. **The prebuilt editor is the default for projects without Rust code.** Why:
  Rust has no stable ABI, so project libraries can't be loaded safely into a prebuilt editor.
- **Runtime features the tools need and games benefit from:** multiple windows, text input and IME, the clipboard,
  drag and drop, idle frame pacing ([Windowing and display](../systems/windowing.md)).

## Every edit is a change record, made through one edit API

- **A document** is a scene file, an asset file the editor writes (a material), an asset's settings file, or a project
  config file. Edits are made to documents; undo, saving and recovery work per document.
- **A change record** is one of: set a field (the target, the component, the field path from reflection, old value,
  new value), create an entity or component (with its serialized content), destroy one (with its serialized content,
  so undo can restore it), reparent. The target is an entity in a document, by UUID, or the document itself (a
  material, a config file). On a placed scene instance, changes to its root's fields and scene properties are stored as
  that instance's overrides; child entities and root components it gains are stored as additions on the instance
  ([Scenes](../world/scenes.md)).
- **What counts as an edit:** a write to a document's saved fields, by any code running in the editor (the editor's
  own tools, Rust extensions, project Rust code, edit-mode TypeScript).
- **Every edit goes through one edit API** (`edit.set(...)`, `edit.create(...)`, ...), which applies the change to the
  editor's world and records it. The inspector, gizmos, the hierarchy, drag and drop, importers' settings and
  extensions all use it. **Undo, redo, unsaved state, crash recovery, live editing and copy and paste are derived from
  the change records,** not written per feature. Why: undo is then always the exact inverse, and no feature needs its
  own command class with its own inverse.
- **No way around it, enforced by types:** Rust code in the editor gets the edited world read-only and writes through
  the edit API (a guard type the write functions are methods on), so a change that is neither undone nor saved can't
  compile. **Edit-mode TypeScript writes through the usual script API** (`light.intensity = 2`); in the editor those
  setters go through the edit API, so scripts get undo for free.
- **Views learn of changes from the records:** after each step is applied, undone or redone, the applied changes go to
  whoever keeps derived state (the hierarchy's tree, thumbnails, the editor link). egui windows such as the inspector
  read the data every frame.
- **Tools that change many things at once group their edits** (`edit.group`). Kept possible: a helper that records a
  brush stroke by diffing before and after, with the terrain tools.

## Undo: one history per document

- **Each open document keeps its own history;** a scene is open while it's open in the editor, an asset or settings
  document from its first edit until the editor closes. **Ctrl+Z undoes the newest change among the open documents,**
  so it feels like one history while closing or reloading a document drops only its own.
- **A step may span several documents** (an override applied back to the instance's own scene); it is undone and
  redone as one.
- **Unsaved state comes from the history:** a document is unsaved when its position differs from the one it had when
  last saved, so undoing back to the saved state clears the mark.
- **One undo step per gesture:** a gizmo or slider drag from press to release, a text field on commit, an edit-mode
  script's writes per callback, a tool's `edit.group("Randomize rotation", ...)`. Each step has a readable name for
  the Edit menu ("Undo Move Crate").
- **Selection isn't undoable;** undo and redo select what they changed, so the user sees it. Kept possible: a
  navigation history.
- **File operations aren't undoable** (create, rename, move, delete assets); deletion goes to the OS's trash.
- **A document reloaded from disk drops its history** (a git checkout changed the file); when it has unsaved edits,
  the editor asks first (reload and lose them, or keep the editor's version). The history doesn't survive closing the
  editor.

## Saving

- **Save writes the whole document atomically:** a crash during a save never leaves a half-written file. The same
  write as save games ([Save games and storage](../world/save_storage.md)). Save, Save All, and a prompt listing
  unsaved documents on close.
- **Unsaved changes are saved before Play,** since play mode starts from the files on disk. Why: what plays is what
  the editor shows.

## Crash recovery through a change journal

- **Crash recovery, when built, replays a journal of the change records on top of the saved files,** not timed
  saves. Why: a crash, an abort, a killed process or a power cut then loses at most the step in progress, and nothing
  has to run at crash time, which `panic = "abort"` and corrupted state make unreliable; timed saves lose minutes or
  overwrite files with changes the user didn't choose to save.
- **The one rule for code built now:** change records are serializable and replay exactly on top of the saved file.
- **When:** not important for the first versions and the first game, but **required before the engine is called
  jam-ready**, which promises no data loss when the editor crashes ([Why the engine exists](../goals/why.md)). Until
  then, an engine assert or a GPU device loss in the editor loses unsaved work.
- **Kept possible:** a recovery mode after a crash during start-up.

## Copy and paste as text

- **What can be copied:** entities with their children and components, a single component, a single field value, and
  scene instances with their overrides.
- **The clipboard holds JSON in the scene file's format.** Why: it works between scenes, editor windows and projects,
  and can be pasted into a text editor or a bug report.
- **On paste:** new UUIDs for the pasted entities; references among the copied entities point at the copies;
  references to entities outside the copy stay pointed at the originals within the same scene and are cleared when
  pasting into another scene, since references never cross scenes; asset references stay if the asset exists in the
  project, else are cleared. A paste that cleared anything says so in one message, with the fields. Assets copied
  between projects keep their UUIDs, so they match. Kept possible: matching assets by path.
- **A paste is one undo step.**

## Live editing in play mode

**Edits made in the editor while play mode runs are ordinary edits** (undoable, saved, kept after Play stops) **and
are also shown in the running game; what the running game itself changes ends with the play-mode process.**

- **While play mode runs, edits to a scene it has loaded are sent over the editor link as the same change records**
  (the entity's UUID chain, the field path, the value) and applied at frame start to every loaded copy of that scene.
  An instance override is applied to the field its scene property names. Always on. Kept possible: a switch to turn it
  off.
- **Hot reload still covers later loads and spawns** ([Assets and project files](../world/assets.md)). Placed instances
  of the edited scene don't follow. A live edit that moves or reparents a static entity is refused with the usual
  error.
- **A change to an entity that play mode has destroyed or never spawned is ignored.** Created and destroyed entities
  are sent too; a created entity starts its scripts by the usual spawn rules.
- **Changes made inside play mode stay there,** including values edited through runtime inspection ([Profiling and
  debugging tools](debug_tools.md)). Kept possible: copying a component's play-mode values back into the editor.
- **Accepted:** live edits aren't on disk until saved, so a scene play mode loads again later in the same run comes
  from the file without them; material and settings edits reach play mode only after a save, through hot reload. Kept
  possible: play mode asking the editor for its in-memory version.

## Collaboration: kept possible

Not designed. The change records with UUIDs and field paths are the form collaborative editors use, so nothing here
closes the door; per-user undo and a server are designed when wanted.
