# In-game UI and text

Game UI (menus, HUD, dialogue), its layout, scaling, drawing and input, plus text and fonts, localization and
accessibility. Editor and debug UI use egui instead.

Terms: a **UI screen** is the entity a tree of UI elements is laid out and drawn in. **UI focus** is the element that
receives keyboard and gamepad input (not the application focus of [Windowing and display](windowing.md)). A **text
key** is a localization string's identity (not a keyboard key). The **UI size** is the player's UI scaling option.
**UI units** are the unit UI elements are measured in.

## Game UI is its own retained system; egui stays for tools

- **A retained UI system of the engine's own** for game UI, built in-house, with crates standing in for layout and
  text ([Why the engine exists](../goals/why.md)).
- **egui is the toolkit for the editor, every tool, and debug and development UI.** Rust code may still draw egui in a
  game for its own debug or tool windows. It isn't the game UI and gets no script API. Why not egui for games:
  immediate mode is code-only (non-programmers can't lay out a menu in an editor), it lacks bidirectional text and
  gamepad input, and its look is built for tools. Accepted: the development console and overlay aren't usable on
  gamepad-only devices until consoles are taken on.

## UI elements are entities, under a UI screen

- **A UI screen is an entity with a UI screen component;** UI elements are its descendants with UI components (a box,
  an image, a text, a button). They are ordinary entities: the hierarchy shows them, the inspector edits them, scripts
  sit on them, signals and tags work on them. Why: one world, no second tree to sync, and no second way to attach
  scripts ([Object model](../world/object_model.md)).
- **Their transform is written by layout:** a UI element's local transform is in its UI screen's UI units, written by
  layout from its layout properties; a write from code is a logged error; the editor hides the transform on UI
  elements and shows the layout properties. The UI screen's own transform is ignored for window and texture targets.
  Layout runs once per frame, after late update and before transform propagation ([Time and the
  frame](../core/frame.md)), so code that sets a text and reads its size sees the new size a frame later.
- **Reusable widgets are scenes** with scene properties (a "menu button" with its text and icon), instanced where
  needed; no separate template concept ([Scenes](../world/scenes.md)).
- **A HUD that stays across levels lives in a scene kept loaded,** since nothing with a transform survives a scene
  switch.
- **Kept possible:** a markup format, as another way to author the same entities; data binding.

## One layout model, CSS-shaped

- **CSS flexbox and grid, with absolute positioning for anchoring.** Anchor presets in the editor ("top left",
  "stretch horizontally", "centre") set the element's absolute insets and auto margins, so the anchor workflow and
  flowing layouts (a list of buttons, an inventory grid) are one system. A pivot other than the box's edges or centre
  is an offset applied after layout, like CSS's `translate`.
- **taffy as the stand-in.** Why: one model where anchors plus containers would be two, and the web model is the one
  more creators already know.

## UI units and scaling

- **UI elements are measured in UI units: pixels of the UI screen's reference resolution,** a third unit next to
  OS-scaled window geometry and physical pixels, apart by type in Rust and scripts.
- **Each UI screen has a reference resolution and a width/height match** (Unity's Canvas Scaler "Scale With Screen
  Size"): designed at 1920×1080, it scales with its target. Physical pixels = UI units × the UI screen's factor,
  computed from the target's size, the match and the UI size.
- **An "integer factor only" option per UI screen** (rounds the factor down), so pixel-exact bitmap fonts and pixel
  art stay sharp on a window target.
- **The UI size is a player setting** (a percentage, default 100 %) applied to every UI screen, for the options menu
  and accessibility; a screen can opt out (a fixed-size minimap) ([Configuration](../core/configuration.md)). Kept
  possible: a separate text size for layouts that reflow, since scaling a whole HUD to 200 % overflows the screen.
- **The OS scale factor isn't applied by default;** a UI screen can opt in to follow it, for app-like games. Kept
  possible: screen density on mobile.

## Where UI draws

- **A UI screen draws to a target, a window or a texture, in that target's `order` together with its cameras:** a HUD
  draws after the scene's cameras, post-processing included; a background menu can draw before
  ([Input and camera](input_camera.md)).
- **The look picks per screen:** a UI screen on a low-resolution render texture gets pixelated UI with the scene; one
  on the window stays crisp at full resolution.
- **World-space UI is a UI screen drawing into a texture** that a material shows (an in-game monitor, a wrist
  display). Kept possible: a UI screen placed directly in 3D.
- **A UI screen on a texture gets its pointer from game code:** code passes "the pointer is at this texel", hit-tested
  at the next frame start like the window's pointer. Why: the engine can't know how a texture maps to the window.
- **Blending** follows the colour pipeline: UI after the last camera blends in sRGB; UI ordered before a camera is part
  of the scene; a texture only UI draws to is sRGB-encoded ([Renderer architecture](../rendering/architecture.md)).

## Input, UI focus and navigation

- **Who sees input first: the development overlay (and egui windows a Rust game draws), then game UI, then actions.**
- **The pointer hit test runs at frame start,** against the layout drawn last frame, the one the player saw. UI
  screens are tested in the reverse of their draw order. No hit test while the cursor is locked, so a locked cursor
  resting over a HUD element never eats fire clicks.
- **What UI consumes:**
  - **pointer presses** on an element marked "blocks pointer", on by default for elements that react; a UI screen has
    a **"blocks input below"** switch for modal menus, consuming every pointer press over it;
  - **pointer capture:** a press the UI consumed captures that pointer and button until release, so moves and the
    release go to the element even outside its box (sliders, scroll bars); per pointer for touch;
  - **releases** only if their press was consumed, so a gameplay button released over a menu never sticks;
  - **the wheel** over a blocking element;
  - **never motion or position;**
  - **a focused text field takes every key press away from actions** (typing "w" doesn't walk); releases still pass.
    Escape, Enter and Tab reach the field as UI cancel, submit and navigate. Kept possible: an on-screen keyboard for
    gamepad-only text entry.
- **Raw device reads still see everything.** Games that read raw input get one query, "is the pointer over blocking UI
  / was this press consumed by UI", so they don't shoot through buttons.
- **Keyboard and gamepad navigation go through actions.** The project template's input asset has a "UI" set
  (navigate, submit, cancel); one project setting names the three actions the UI uses, defaulting to that set, so the
  engine finds them by reference, not by a fixed name. A missing action is logged like an unknown action name.
- **One UI focus.** Neighbours are computed from the layout (the nearest element in that direction), with explicit
  overrides per element; navigation stays inside the focused element's UI screen. Each UI screen names its first UI
  focus; while an enabled UI screen names one, focus is never nowhere: the most recently enabled such screen takes it,
  so closing Options over Pause hands focus back to Pause. A click on a focusable element moves UI focus to it. The
  focus indicator shows after keyboard or gamepad navigation and may hide while the pointer is used.
- **Interactions are signals** (`clicked`, `valueChanged`), emitted by the engine at frame start and delivered to
  scripts at the settle point ([Signals](../scripting/signals.md)). Rust reads the same fact as the widget's state
  (`Button::clicked()`), **true for the frame of the interaction**; reading it in the fixed update is a logged error,
  as raw device reads there are.
- **Pause menus** are UI screens on entities marked "runs while paused"; without the mark, a menu script awaiting a
  signal hangs while paused ([Time and the frame](../core/frame.md)).
- **The manual teaches disabling the gameplay action set while a menu has UI focus,** so "Resume" doesn't also jump.
- **Kept possible:** per-local-player UI focus (one shared focus driven by every local player's UI actions is enough
  now).

## Text and fonts

- **Full Unicode text in every build: shaping, bidirectional text and line breaking,** so Arabic, Hebrew, Indic scripts
  and ligatures are right, at about 0.45 MB on the web. **Dictionary word breaking** (about 2.4 MB) is a build feature a
  project enables for the locales that need it, mostly the South-East Asian scripts.
- **Crates:** parley for text layout as a stand-in; **harfrust is kept** for shaping, like naga. Why: writing a
  HarfBuzz-class shaper would be a project of its own.
- **Fonts are assets shipped with the game; no system fonts** (none on the web or consoles). A project names a
  **fallback chain** of fonts (Latin, then CJK, then emoji), overridable per locale, so a missing glyph never shows a
  box when a font in the chain has it.
- **A font can be drawn pixel-exact** (bitmap fonts with nearest filtering) as well as smooth at any size; how glyphs
  are drawn is otherwise implementation.
- **Kept possible:** rich text markup (bold, colour, inline icons), colour emoji, text along paths, IME on the web.

## Localization

- **A localizable text type,** separate from a plain string: a text property (a component field or an `@export`
  property) holds a text key, and the editor offers the string tables' entries. Plain strings stay plain (names,
  paths, debug text). This is in every scene file, so it is decided now.
- **The text type holds a text key with arguments, or a literal string:** "Score: {score}" is a key with an argument;
  a player's name is a literal, never translated.
- **String tables are assets: text keys to translations per locale.** A text key's identity is a UUID with a readable
  `Name`, so renaming it never breaks editor data. Code looks texts up by the readable name (scripts pass strings); a
  lookup by an unknown name is logged in development builds.
- **The current language is a player setting, defaulting to the OS locale.** Switching it at runtime re-resolves every
  text on screen.
- **Plurals and gender are required.**
- **The editor stays English for now.** Kept possible: editor localization, per-locale assets beyond fonts (voice,
  textures with text).

## Accessibility

- **Every built-in widget can report its semantics** (a role, a label, a value), derived from its type and text, so a
  screen reader can be added without changing scenes. Nothing is stored now; an optional override component comes with
  screen readers.
- **Screen readers on native platforms when built** (AccessKit's adapters). Kept possible: the web.
- **Player options:** remapping and the UI size now; subtitles, colour-blind filters and text-to-speech later.
  Text-size minimums go into the documentation and templates.
- **The editor gets AccessKit through egui** on native platforms; the runtime's own egui integration wires it.

## Editor-wired connections: kept possible

Wiring a button's `clicked` to a method in the inspector stays a door until visual scripting is designed: it needs
callable methods listed for the inspector and method references in scene files, and a renamed method would break the
connection silently. Wiring menus now: an exported entity reference set in the inspector says which button belongs to
which menu, and one line of code reacts: a script connects in `start()`
(`this.connect(this.resume.getComponent(Button).clicked, ...)`), a Rust system checks the button's state. Saved
connections inside one scene are added at whichever comes first: the first game showing the need, or the design of
[visual scripting](../scripting/visual_scripting.md).
