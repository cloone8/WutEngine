# Signals

How objects tell each other that something happened, and how engine events reach scripts. Signals are synchronous
calls; waiting is in [Script API](api.md).

Terms: an **event** is something that happens (input, a window change, a lifecycle moment); a **signal** is the typed
field scripts connect to. Engine events reach scripts through signals. The term is Godot's; in web JavaScript
"signals" usually means reactive state, a collision accepted because the alternatives are taken.

## A signal is a typed field

A signal is a field declared on a script class, a Rust component or an engine service object (such as `game`): in
TypeScript `opened = new Signal<[by: Entity]>()`, in Rust a `Signal<T>` field. A sender announces that something
happened without knowing who listens; a script that knows its target can still call it directly.
Why: one concept, and as declared fields the generated `.d.ts` types their arguments.

## A connection ends with its script component

- **`this.connect(door.opened, (by) => ...)`** connects a handler. The connection ends by itself when the listening
  script component is removed or destroyed, its scene unloaded, or the sender is gone; `connect` also returns a
  connection with `disconnect()`.
  Why: the same rule as waits, "whatever goes through `this.` ends with this component"; forgetting to disconnect in a
  destroy handler is a classic trap.
- **A signal is awaitable:** `await this.wait(door.opened)` settles on the next emit, when the handlers of that emit
  run ([Script API](api.md)).

## Emitting from scripts is immediate

- **`this.opened.emit(by)` calls the handlers right away,** before `emit` returns, in connection order.
- **The handler list is fixed when the emit starts:** handlers connected during the emit aren't called; handlers
  disconnected or destroyed during it are skipped. Nested emits are allowed. A handler that throws is logged and the
  others still run.
- **Handlers are calls:** they run whether or not the listener's entity is paused; only waits are held while paused.
- Why immediate: the sender can rely on its listeners having reacted, and chains don't spread over steps. It doesn't
  affect parallelism: scripts emit only in the script step, which never overlaps Rust systems, and script emits call
  only script handlers.
- **Kept possible:** opt-in deferred connections.

## Signals and Rust: Rust emits, only scripts connect

Rust systems run in parallel on worker threads and never during the script step, and boa's VM can't leave its thread.
So a Rust system can't call script handlers while it runs.

- **Rust can emit signals; only scripts connect handlers.** A Rust component's signal (`Health.died`) is emitted from
  a system through its command queue.
- **One delivery point per frame:** Rust and engine emits are delivered on the main thread at the settle point, in
  the update phase after the Rust systems: first the queued emits, in a deterministic order (system registration
  order, then emit order), then the engine's promises settle, then the scripts' `update` callbacks. An emit made in
  fixed update, late update or a phase without scripts waits for the next settle point. Lifecycle signals are the
  exception (below).
- **A queued emit reaches the connections that existed when it was made,** as a script emit fixes its list. Why: a
  system that emits `died` and queues the despawn would otherwise lose its signal, since the commands apply before
  delivery. The sender may therefore be destroyed when the handler runs; its handles then throw.
- **Scripts to Rust:** method calls into Rust, or component data systems read through their queries. **Rust to
  Rust:** components and queries.
- **No Rust handlers.** Why: a Rust closure run in the script step would be Rust gameplay code outside the scheduler,
  sequential and without declared access, a second way to write Rust gameplay next to systems. Rust runs in the script
  step only as the engine API scripts call. Lifecycle hooks are the exception (below).
- **Kept possible:** Rust systems reading a signal's emits as data, with declared read access, if a real use appears
  (without silently dropping unread messages).

## No global event bus; engine events as signals

- **Across scenes, a signal on a global entity found by type** does the job ([Scenes](../world/scenes.md)); a template
  "game events" script shows it. Why: a bus with string names finds typos only at runtime and breaks every use on a
  rename, and a second, queued message system would be two concepts where one does.
- **Lifecycle events reach scripts as signals on the `game` service object:** `game.quitRequested`, `game.quitting`,
  `game.backgrounded`, `game.foregrounded`; "started" is the start scene's first callbacks
  ([Lifecycle and memory](../core/lifecycle.md)).
  - **They run synchronously when the platform event arrives,** outside the frame's phases. Why: mobile suspend stops
    the loop, so there may be no next settle point, and backgrounded exists to save in the few seconds the platform
    gives.
  - **Quit requested is cancelled through its argument** (`request.cancel()`), before the handler's first `await`: a
    "quit without saving?" dialog cancels first and calls `quit()` later.
  - **Quitting handlers run synchronously;** file writes they start are waited for by the flush step; code after an
    `await` never runs once quitting.
  - **On fatal error stays Rust-only:** the engine is degraded then, so no script runs.
  - **Rust keeps registered lifecycle hooks** for the same events, including one for quit requested that can cancel
    it, so a game written only in Rust has them. They are the exception to "only scripts connect": they run at the
    platform event, not inside a phase.
- **Engine-wide events are signals on engine service objects, added per service when it is built** (application
  focus, [Windowing and display](../systems/windowing.md); asset hot reload, [Assets and project
  files](../world/assets.md)). Scripts connect as usual; delivery is at the settle point; Rust code reads the same fact
  as data ("changed this frame", or a list where it can happen several times per frame). Scene loads aren't signals:
  `load` already returns a promise.
- **Configuration keeps its rule:** readers ask whether a value changed this frame; no change callbacks
  ([Configuration](../core/configuration.md)). Kept possible: a script `changed` signal on configuration sections.
- **Collision and trigger callbacks stay callbacks** ([Physics](../systems/physics.md)).

## Connections authored in the editor: kept possible

Wiring a button's click to a door's `open` in the inspector, without code, helps non-programmers, but needs method
references in scene files and a list of callable script methods. Not now. Signals are declared fields, so a saved
connection (signal to entity and method, inside one scene) can be added to the scene format later; it comes with
whichever is first, the first game showing the need or the design of [visual scripting](visual_scripting.md). Until
then, an exported entity reference plus one line of code wires a menu ([In-game UI and text](../systems/ui_text.md)).
