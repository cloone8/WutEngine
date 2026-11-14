# Input and camera

How games read input (actions, "just pressed" across fixed steps, the per-tick record, keys, mouse, gamepads) and how
cameras turn into views. Windows, cursor modes and text input are in [Windowing and display](windowing.md); how UI
takes input before actions is in [In-game UI and text](ui_text.md).

## Actions are the main API; raw devices stay available

- **One input asset per project,** authored in the editor and editable as a file, shaped like Unity's Input System: it
  holds action sets; each set holds actions; **bindings belong to the action.** Each action has a `Name` and a value
  type: **button, axis or 2D vector** (a 3D vector can be added without breaking anything). Why: the shape most
  creators know, and the CLI path works on the file.
- **Composite bindings** turn several raw inputs into one value: two buttons into an axis (A/D), four into a 2D vector
  (WASD, a D-pad).
- **Scripts get the action names in the generated `.d.ts`,** so a misspelt action is a type error there.
- **Unknown action names** (Rust, and scripts reading a name at runtime): in development builds, polling an action
  that isn't in the input asset logs an error once per call site and returns "not pressed" or zero
  ([Error handling](../core/errors.md)). No generated Rust module of action constants.
- **Actions can also be defined in Rust code,** as values of the same type the asset loads into, for games made
  without the editor. Actions defined only in code aren't in the `.d.ts`.
- **Polling first:** `pressed`, `justPressed`, `justReleased`, `value` per action. Events on actions can be added later
  on the same state (as signals).
- **Raw devices stay available** (keys, mouse, gamepads, touch, text input). Why: the editor, the development console
  and tools run on the runtime and need them, and a jam game may skip actions (reading keys per frame).
- **Interactions come later** (press, hold, tap, multi-tap; hold-or-toggle among them). The binding format leaves room
  for an optional interaction per binding; until then a game times a hold itself.
- **Kept possible:** reading one action across several sets, with priorities between sets.

## "Just pressed" is answered for the phase it's called in

**In the fixed update, `justPressed` means "since the previous fixed step"; everywhere else, "since the previous
frame".** A tap shorter than a step or a frame still counts once. Raw devices are read in per-frame code and get
per-frame edges.
Why: at 60 Hz fixed steps a frame runs zero steps on a fast display and several after a hitch; "pressed this frame"
would miss taps or jump three times. One function everywhere matches `delta()`, which already returns the fixed step
inside the fixed update ([Time and the frame](../core/frame.md)). A press read by both a per-frame and a fixed-step
script is seen once by each.

## The fixed update reads a per-tick input record

- **At each fixed step the engine fills one record per local player:** every action's value and its edges for that
  step. Action reads in the fixed update read that record, never the devices. It is the per-tick data a replay plays
  back and a client would send ([Multiplayer](../goals/multiplayer.md)); nothing is recorded, stored or sent now.
- **Only actions can be read in the fixed update.** A raw device read there is a logged error (once per call site,
  "not pressed" or zero, the game continues), with a message saying what to do: add an action, or read it per frame.
  Why: everything fixed-step gameplay reads is then in the record, so replays and networking can't silently miss
  input; starting strict breaks no game later, the reverse would.
- **Not a full guarantee:** gameplay started from per-frame code escapes the record. The documented rule: simulation
  in the fixed update reading actions; look and feel (camera, UI, effects) per frame.
- **Analog values in the record:** absolute inputs (stick, trigger) are the value when the step starts; relative
  inputs (mouse motion, scroll) are the sum since the previous step, so nothing is lost or counted twice.
- **Kept possible:** games writing the record themselves (bots, automated tests); game-supplied values per step (an
  aim direction) when replays and networking are designed.

## Action sets and rebinding

- **Action sets** ("Gameplay", "Menu", "Vehicle") are enabled and disabled by game code; all enabled sets are active
  together, with no priorities or consumption between them.
- **Enabling and disabling sets:** an action in a disabled set records no edges; disabling a set releases its
  actions; an action enabled while its input is held reads as pressed, but not "just pressed" until pressed again.
  Why: otherwise a menu's "Resume" bound to the same button as "jump" would also jump.
- **Rebinding at runtime** through an API an options menu uses ("wait for the next input and bind it"), saved as a
  sparse patch in the player settings ([Configuration](../core/configuration.md)): only changed actions, falling back
  to the project's bindings (so a changed default reaches players who never touched that action), unknown entries
  kept. The engine reports conflicts **within a set** and answers "which actions use this input?"; the game decides
  what to do.

## Keyboard keys

1. **Bindings store the physical key position by default,** with an optional per-binding **"by character"** mode for
   keys whose letter is the point ("I for inventory"). Why: positional bindings work on non-QWERTY layouts out of the
   box.
2. **WutEngine's own key type,** not winit's: positions named after US QWERTY and defined by the USB HID usage. winit's
   type converts one-to-one on desktop and the web; other backends fill it from their own sources. Games and saved
   bindings never see winit's type. Why: future platforms aren't locked out by one library's gaps.
3. **Labels follow the player's layout,** with the US name as the fallback; non-Latin layouts show the key's Latin
   letter.
4. **Shortcuts are an editor and tools convention, not an engine concept:** the editor's shortcuts follow the layout
   (Ctrl+Z is the key labelled Z), with positional exceptions (WASD flythrough, QWER tool modes). Games that want
   shortcut-like keys use the "by character" mode, which falls back to the position on non-Latin layouts.
5. **Text input is separate:** text and IME events, never actions ([Windowing and display](windowing.md)).
6. **Keys come from window events,** which carry the physical key, the logical key and the text in one event.

## Gamepads

- **Buttons by position** (south, east, west, north), with labels per controller family for display. Games that care
  about "confirm" conventions bind both or check the family.
- **gilrs as a stand-in behind WutEngine's own gamepad API.** Games see only the engine's API (buttons by position,
  sticks, rumble, connect and disconnect events), so the backend can be replaced without touching a game. Why: it
  covers desktop and the web now; Android needs its own backend later, and SDL3's gamepad layer is the natural desktop
  upgrade if gilrs falls short.
- **Dead zones per binding,** radial by default for sticks.
- **Device loss:** a gamepad disconnecting is an event the game sees; what to do stays with the game. Scripts get a
  signal on an engine service object, Rust data that can hold several connects per frame
  ([Signals](../scripting/signals.md)).
- **Rumble** through the player's or device's API.

## Mouse

Each source is read for what it's good at, never summed.
- **Buttons, wheel and position from window events:** logical buttons (the OS's left-handed swap respected), clicks
  with their window and position, on every platform; the wheel in lines (trackpad pixels converted). The position
  lives in input, together with the window it's over (`None` outside every window), in that window's physical pixels
  from its top left.
- **One mouse motion value:** raw motion while the cursor is locked (mouse look), cursor movement otherwise (UI
  dragging, edge panning); in the fixed update summed per step. Where raw motion doesn't exist, a locked cursor falls
  back to cursor movement where the platform reports it. Why: a creator never has to choose between two deltas.
- **Raw motion comes from winit everywhere it exists** (none on mobile for now).
- **No engine "raw mouse input" setting;** added if testing shows a need. **Sensitivity** is a per-binding scale or
  the game's own option; units are not normalised across platforms.
- **One keyboard-and-mouse device:** keyboard and mouse are one device for player assignment; several mice or
  keyboards are not told apart. Why: per-device identity exists on few platforms. Two players on one keyboard use
  different keys through bindings or sets. Kept possible: a Windows-only split.

## Other inputs

- **Several local players:** each has its own action state and per-tick record; devices are assigned to players
  through an API; a single-player game gets every device by default.
- **Android's back button** arrives as a key and binds to an action like any key.
- **Touch** arrives as pointers; gestures are later.
- **Accessibility:** full remapping covers most of it; hold-or-toggle as a binding option is a small later addition.

## Cameras

- **A camera is a `Camera` component that produces a view.** Its entity's transform is the viewpoint (looking along
  +Z); projection perspective or orthographic; a near plane; the far plane infinite, with an optional far distance for
  culling; a target (a window, the main one by default, or a render-texture asset); a normalized viewport rectangle
  (split screen, picture in picture); an `order`. Exposure is EV100 on the camera; tonemapping belongs to the target
  ([Renderer architecture](../rendering/architecture.md)). Other views (shadow cascades, probe faces, captures) don't
  need a camera. Why a component and not a plain view struct: something to place, select and preview in the editor.
- **Field of view and orthographic size name their axis:** `FieldOfView::Vertical(degrees)` or
  `FieldOfView::Horizontal(degrees)`; `OrthographicSize::Height(units)` or `OrthographicSize::Width(units)`, the
  **full** height or width. Defaults: vertical and height. The chosen axis stays fixed when the viewport's shape
  changes (vertical: wider screens see more to the sides; horizontal: taller screens see more above and below, for
  portrait games), so the enum is kept and converted with the current aspect each time the view is built. The enums
  are `#[non_exhaustive]`, leaving room for "fix the smaller axis".
- **Every enabled camera renders.** Switching cameras is enabling one and disabling the other.
- **`order` is per target:** among cameras on one target, lower first; a later camera draws over an earlier one (an
  overlay doesn't clear colour). Equal orders on one target are a development warning and fall back to hierarchy
  order, so the result is still deterministic.
- **Across targets the renderer orders by dependency,** never by `order`; cycles read the previous frame
  ([Renderer architecture](../rendering/architecture.md)).
- **No "main camera" tag.** Code that needs a camera takes it as an argument; a helper returns the camera that draws
  last to a window.
- **No camera:** the target shows its clear colour and, in development builds, a "no camera renders to this window"
  message.

## How cameras move: the engine has no opinion

- **The one engine rule:** a camera's view is its entity's transform as displayed this frame, read in render
  preparation after all updates. A camera parented to an interpolated entity uses that displayed pose, so it never
  jitters against what it is attached to ([Time and the frame](../core/frame.md)).
- **Documentation and templates, not engine rules:** a camera that follows by code moves in the late update; mouse
  look is applied per frame, never in the fixed update; code that follows an interpolated entity without being
  parented reads the target's displayed pose, or it jitters.
- **Kept possible:** late latching of the camera; this rule and the per-view buffer keep it open.

## Screen and world conversions

- **`screenToRay`, `worldToScreen` and `screenToWorld` on the camera.** Screen positions are in the **target's**
  coordinates, from its top left, so a mouse position passes straight in; the camera accounts for its own viewport
  rectangle. Rays and points are `WorldPos` in Rust and absolute f64 in scripts ([Game size and
  precision](../goals/game_size.md)). Why: they can't be game code without reimplementing the projection.
- **Units are the target's own:** the window's physical pixels for window targets, texels for render-texture targets.
  For rays the unit cancels out. A click on an in-game monitor becomes the hit's texture coordinates times the
  texture size.
