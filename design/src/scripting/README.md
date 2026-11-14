# Scripting

How game logic is written without Rust: the language, the script engine, threading, compiling TypeScript, hot reload
and debugging. The API scripts see is in [Script API](api.md), communication between objects in [Signals](signals.md).

## Who scripts are for

**Scripts are the beginner and intermediate tier; Rust is the advanced tier.**
Why: simple defaults with the advanced APIs exposed ([Principles and constraints](../goals/principles.md)). Rust code
from a programmer still reaches every teammate, because every project compiles its own player and, with Rust code,
its own editor ([Building and shipping](../editor/shipping.md), [Editor](../editor/README.md)).

The constraints scripting must meet: beginner-friendly; no JIT; runs on desktop, `wasm32-unknown-unknown` and later
consoles; hot reload in the editor; explicit registration of what Rust exposes; errors shown in the editor with
their locations; text files that merge in version control.

## Language and engine

- **No scripting language or VM of our own, and no poorly supported dependency.**
  Why: a language and its tooling are a project of their own; a weak dependency becomes ours to maintain.
- **Language: TypeScript, shipped complete with full type checking**, no shortcuts.
  Why: widely known, with real types and editor tooling; beginners get errors before running.
- **Script engine: boa**, a pure-Rust JavaScript interpreter.
  Why: no C toolchain for any target (the web included), healthy activity, and no JIT, so it fits consoles and iOS.
  Accepted costs: roughly 5–6× slower than QuickJS, and yearly releases with API changes.
- **The script API is generated from the engine's registration and reflection schema**, not written in boa's binding
  style. Why: the script engine stays swappable.

## Threading

- **All gameplay scripts share one VM on one thread.** Engine work and Rust systems run around the script step,
  never at the same time ([Time and the frame](../core/frame.md)).
  Why: boa's context can't be shared or moved between threads, and several VMs splitting the scripts would break
  references between scripts in ways a beginner can't understand.
- **Kept possible:** script jobs on worker VMs later, with data copied in and results copied back (Web Worker style;
  sequential on the web).
- **Script data:** a script attached to an entity declares `@export` properties ([Reflection](../world/reflection.md));
  there is one instance of each script class per entity ([Object model](../world/object_model.md)).

## TypeScript to JavaScript

boa runs JavaScript, so `.ts` is compiled to `.js` in the editor and at build time; shipped games contain only
JavaScript.

- **The editor bundles the pinned native `tsc` (TypeScript 7) per platform** and uses it for both type checking and
  JavaScript emit. Why: one tool, no Node, maintained by Microsoft. Costs: about 25 MB per editor download, and an
  engine update can bring a stricter TypeScript that reports new errors in existing projects.
- **Scripts are written in an external code editor** (VS Code and the like). The engine generates `tsconfig.json` and
  the `.d.ts` files of the engine API into the project, so the code editor's own TypeScript support gives completion
  and errors without a plugin.
- **Type errors block entering play mode and building.** Why: like compile errors; a project that doesn't type-check
  isn't run.
- **Diagnostics come from `tsc`'s command-line output** (file, line, column, code, message).
- **Source maps** from `tsc` map runtime errors back to `.ts` lines.

**Generated `.d.ts` without the editor: `webuild types`.** The project's play-mode player has a mode that runs the
game's registration, writes `tsconfig.json` and the `.d.ts` files into the project, and exits without opening a
window; `webuild types` compiles the project and runs that mode. Every `webuild` build runs the same step before
type-checking, and the editor uses it too: one generator for every path. The same run writes the script API
reference ([Documentation](../editor/documentation.md)).
Why a run step: registration is explicit code in the game's entry point, so the full list of exposed types only
exists when the compiled project runs; Rust has no compile-time reflection a tool could query. Generated files are not
committed, since a CLI-only user changing Rust code would leave them stale.

## Hot reload

What happens when a script is saved while the game runs from the editor. Play mode is a separate process
([Editor](../editor/README.md)): the editor recompiles with `tsc` and sends the new JavaScript to the play-mode player,
which swaps it in.

- **Level 1 first: changes apply at the next Play.** Simple and always correct; needed in any case.
- **Level 2 later, as a setting that is on by default:** swap code while playing and keep the state. Each existing
  script object is pointed at the new class's methods (`Object.setPrototypeOf`); field values stay, fields new in this
  version get their defaults. Limits:
  - callbacks already registered (signal handlers, pending waits and promises) keep running the old code until they
    are registered again;
  - top-level variables in a changed file are reset when the file runs again;
  - a class whose old or new version declares JavaScript `#private` members can't be swapped: that class falls back to
    "restart needed" (detected by parsing the emitted JavaScript), while the rest of the reload goes ahead.
- **The editor says plainly when a reload couldn't keep everything** and offers a restart, rather than failing
  silently.
- **`#private` fields are allowed,** and the editor shows a warning-level hint that TypeScript's `private` keeps hot
  reload working. Why: each evaluation of a class body creates new private names, and JavaScript offers no way to list
  or copy them, so they can't be carried over; forbidding `#` would remove a standard language feature.

## Debugging

- **For now, no breakpoints.** Debugging is `console.log` into the editor console, runtime errors with stack traces
  mapped to `.ts` lines through source maps, and the inspector showing live script fields. What a script error does
  to the callback and the frame, the loop limit and "pause on error" are in [Error handling](../core/errors.md).
  Why: boa has no debugger API.
- **Kept possible:** debugger hooks contributed upstream to boa in small steps, with the editor or player hosting a
  Debug Adapter Protocol server so VS Code can debug scripts. Risk: acceptance depends on boa's maintainers; a refusal
  would leave us holding a fork.
