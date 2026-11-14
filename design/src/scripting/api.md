# Script API

How Rust data and functions look from TypeScript, and how scripts wait for things. The `.d.ts` files are generated
from the registration, never written by hand ([Scripting](README.md)).

## Shape of the API

- **Engine objects are handles; math and value types are immutable copies.** Entities, components and assets are
  small script objects pointing at Rust-owned data; every read or write goes through to Rust. `Vec3`, `Quat`, `Color`
  and similar are copied into script values with `readonly` fields and `with…` helpers.
  Why: a call into Rust for every `.x` would cost a boundary crossing each time.
- **`readonly` fields on value types.** With copies, `transform.position.x = 5` would silently change a copy and
  nothing would move; with `readonly` in the generated `.d.ts` it is a type error, shown in the code editor and the
  WutEngine editor. The correct forms are `transform.position = transform.position.withX(5)` or a new `Vec3`.
  Why: turn a silent trap into a compile error. Mutable value types are therefore not offered.
- **Functions are exposed by explicit registration, with their types.** One registration produces both the boa
  binding and the `.d.ts` entry. Why: the reflection schema describes data, not functions, and one source keeps the
  binding and the types from drifting apart.
- **Angles:** every angle parameter names its unit (`fovDegrees`), and conversion helpers exist; rotations themselves
  are quaternions ([Conventions and names](../core/conventions.md)).
- **A handle to a destroyed entity throws a clear error**, instead of a "fake null" that silently does nothing. The
  engine catches the throw at the callback and logs it, so it never crashes anything; Rust callers get a logged error
  and a neutral value ([Error handling](../core/errors.md)).
- **A destroy callback on script components,** called when the component is removed or its entity destroyed.
  Why: a component's pending waits never settle once it is gone (below), so their `finally` blocks don't run, and
  cleanup that must happen needs a place.
- **Pixel units stay apart in scripts.** OS-scaled window geometry and physical pixels are distinct types in the
  generated `.d.ts`, as in Rust ([Windowing and display](../systems/windowing.md)), each with a **brand** (a type-only
  tag such as `{ readonly __unit: "physical" }`, erased at compile time) and explicit conversion functions
  (`window.toPhysical(size)`). Why: TypeScript's structural typing would otherwise let a script compare the cursor with
  `window.size()` and be silently wrong at scale 2. UI units join them ([In-game UI and text](../systems/ui_text.md)).

## Design for JavaScript, not for boa's current speed

- **The script API follows common JavaScript API design,** not boa's current performance. Why: boa is largely
  unoptimized and expected to get faster; the bindings are generated, so the engine also stays swappable.
- **A documented script budget per boa version,** stated as current numbers (in the first version measured, roughly
  340–380 moving script objects per millisecond native and about half that in a browser). Heavy per-object motion
  belongs in Rust components; scripts do game logic. Why: immutable copies allocate, and allocation drives boa's
  garbage-collection pauses; creators should know the limit up front.
- **Convenience methods on handles** for common operations (`transform.translate(v)`). Why: idiomatic API; that they
  skip an allocation and a crossing is a side benefit.

## How scripts wait

Terms: a **continuation** is the code after an `await`, run when the awaited promise settles. The **settle point** is
the one moment per frame where the engine settles its own promises (below).

- **Standard `async`/`await` on promises,** no engine coroutines or generators. Why: TypeScript ships complete, and
  promises are what web developers know.
- **Waits are methods of the script component:** `await this.waitFrame()`, `await this.waitSeconds(2)` (game time),
  `await this.waitRealSeconds(2)` (real time). Why: a wait must know its entity (to stop while it is paused), and after
  an `await` there is no "current script" the engine could infer it from. One rule for non-programmers: whatever goes
  through `this.` ends with this component (waits here, connections in [Signals](signals.md)).
- **`this.wait(x)` binds any other promise or a [signal](signals.md) to the component** (an asset load, a scene load,
  the next emit of a signal). It adds only the component rules below and otherwise settles when `x` does: right after
  the callback that settled it for a promise settled by script code or a script emit, at the settle point for an
  engine promise or a Rust emit.
- **A wait for seconds settles at the first settle point after the time has passed.** Repeating timers are a loop
  around a wait.
- **No `setTimeout` or `setInterval` in game scripts** (absent from the generated `.d.ts`, so a type error).
  Why: they run on wall time with no component, so they would ignore pause and time scale, outlive their entity, and be
  a second timer system next to the waits. `queueMicrotask` stays. Cost: an npm library that uses `setTimeout` needs a
  shim.
- **Loading returns a promise in scripts** ([Assets and project files](../world/assets.md)).
- **Kept possible:** a wait for the next fixed step, cancelling a single wait early (`AbortSignal`), "wait until a
  condition" as an API (a documented loop over `waitFrame` does it).

## When continuations run

- **Script-settled promises continue as soon as the script stack is empty:** after every callback the engine makes
  from outside the scripts (`update`, a collision callback, a handler of a Rust or engine emit, a continuation), it
  runs the continuations that are ready, as browsers drain microtasks. A continuation never runs in the middle of
  another callback; handlers of a script emit are part of the emitting callback, so no drain runs between them.
- **A cap per drain:** the drain stops at the cap and logs a script error; the remaining continuations run at the next
  drain. Why: a chain of continuations that never ends must not hang the frame.
- **The engine settles its own promises at the settle point: in the update phase, after the Rust systems and before
  the scripts' `update` callbacks**, in the order the waits were made, right after queued Rust and engine signal emits
  are delivered ([Signals](signals.md)). That covers waits, loads and every other engine promise. Why: code after
  `await this.waitFrame()` then runs like code at the top of `update`, seeing this frame's input and Rust results, and
  late update (camera follow) sees what it did ([Time and the frame](../core/frame.md)).
- **Order is deterministic:** waits settle in creation order; continuations run in the order their promises settled.
- **The engine runs its own job executor** instead of boa's default: it decides when continuations run and holds the
  cap. A throwing continuation becomes a rejected promise, logged like any unhandled rejection; errors that reach the
  executor itself (a throwing `queueMicrotask` callback, a failing Rust future) are isolated per job and logged.
  Why: boa's default executor drops all pending work on the first error.

## Pending work when a component goes away

- **When a script component is removed or destroyed, or its scene unloaded, its pending waits never settle:** the
  engine drops their resolving functions and the suspended `async` functions are garbage-collected. For `this.wait(x)`
  the engine also removes its listener on `x`. Code after the `await` simply never runs.
  Why: a cancellation error would run `catch` blocks written for real errors, and code after them would run on a
  destroyed entity where every handle throws. Never settling is one sentence: "when an entity is destroyed, its
  scripts stop, waiting code included".
- **Cost:** `finally` blocks after a component's wait don't run when it goes away; cleanup goes in the destroy
  callback.
- **While the entity is paused** its waits don't settle; real time keeps counting for `waitRealSeconds`, game time
  doesn't; they settle at the first settle point after the entity runs again.
- **A promise not bound through `this`** (a plain `await load(...)`, a script's own `new Promise`) is ordinary
  JavaScript: it continues when it settles, and touching the destroyed entity then throws and is logged. The manual
  teaches `this.wait(...)` in entity scripts. Awaiting something that never comes hangs silently.
- **Play mode** ends with its process, so no async work outlives it.
- **Hot reload:** pending continuations keep the old code ([Scripting](README.md)).
- **Save games save state, not pending flows:** a suspended `async` function can't be serialized, so a wait in
  progress restarts from the saved state ([Save games and storage](../world/save_storage.md)).
