# Time and the frame

How a frame is ordered, how Rust systems and scripts are scheduled inside it, how the frame is paced against the
display, and the clocks game code reads.

## The frame's phases

A fixed, documented order of phases, and **each phase is a barrier**: everything in it finishes before the next phase
starts. User code sees a strictly sequential frame. Parallelism the user can't observe lives inside engine steps
(physics, transform propagation, culling, render recording), each starting and ending at a fixed point, and in
background work that doesn't touch the world, whose results enter the world at fixed sync points.
Why: user code can then reason about the frame as if it ran on one thread, which it does on the web.

0. **Frame start:** platform events and input (the development overlay takes the input it wants first); results of
   background work enter the world: loaded assets, finished scene loads (a scene switch takes effect here), GPU
   readbacks, editor-link messages, configuration changes. Then the game side of audio advances each sound's time,
   and sounds that ended queue their `finished` for the settle point ([Audio](../systems/audio.md)).
1. **Fixed-update loop, 0..n steps** at a fixed rate. Per step: fixed update (Rust systems, then scripts), transform
   propagation, physics step, collision and trigger callbacks.
2. **Update** (Rust systems, then scripts). Between the two is the **settle point**: queued Rust and engine signal
   emits are delivered, then the engine's promises settle ([Signals](../scripting/signals.md),
   [Script API](../scripting/api.md)).
3. **Animation** (once it exists): writes local transforms.
4. **Late update** (Rust systems, then scripts): camera follow, overriding animation. Then **UI layout** writes UI
   elements' transforms ([In-game UI and text](../systems/ui_text.md)). Then, in development builds, **the
   development overlay's UI**, on the main thread, so its inspector edits go through propagation in this frame
   ([Profiling and debugging tools](../editor/debug_tools.md)).
5. **Transform propagation.**
6. **Render preparation:** the one sync of changes into the retained GPU scene; in parallel, audio sources and the
   listener read their displayed poses and the frame's batch goes to the mixer.
7. **Render graph:** build, record, submit, present. The development overlay draws last.

A headless build still runs every phase; only the backend work in phases 6 and 7 is skipped
([Lifecycle and memory](lifecycle.md#headless)).

Rules within the frame:

- **Rust systems first, then scripts,** within a phase. Why: scripts are the game logic on top and see this phase's
  Rust results. Rust code that must run after scripts uses the next phase.
- **Script order within a phase is hierarchy order by default** (depth-first, top to bottom as in the editor; entities
  without a transform in their list's order). Where it matters, a script class declares an explicit order in code,
  lower first.
- **A script's `start()` runs right before its first update.** Creation and destruction follow the
  [Object model](../world/object_model.md).
- **No frame pipelining for now** (simulating frame N+1 while rendering frame N). Why: it adds a frame of latency and
  more complex code. Kept possible: the retained render scene already decouples rendering from the ECS.

## Parallel equals sequential

- **The engine only parallelizes work whose result doesn't depend on order:** the same result whatever the thread
  count, and from run to run, on the same build and machine, float reductions included; not identical bits across CPU
  architectures. It covers engine code; user code that shares state through locks, atomics or channels is the user's
  responsibility. How the job system guarantees it: [Jobs and async](jobs.md).
  Why: the web runs everything sequentially, so a parallel result that could differ would give bugs on one platform
  only; reproducible runs with a configured seed rely on it too.

## Rust scheduling

Rust is the advanced tier, so Rust code may modify other entities as long as its data access is declared:

- **Systems declare their data access, enforced by their types:** a system only receives what it declared, so
  undeclared access is a compile or registration error, not a silent race.
- **Systems that don't conflict run in parallel.**
- **Conflicting systems run in explicit before/after order where given, otherwise in registration order,** which is
  deterministic; the result equals sequential execution in that order. There is no "ambiguous order" warning; a
  schedule view in the development overlay shows sets and orderings.
- **A system that writes only the entity it processes is spread over all cores automatically;** small workloads stay
  on one thread.
- **Cross-entity writes are allowed,** at the cost of that system's own parallelism, or go through commands.
  Structural changes are commands ([Object model](../world/object_model.md)).

Why: parallel should be the easy path for Rust programmers, not an opt-in, and declared access with a defined order
gives the same determinism as a restrictive "own entity only" model. The scheduling idea is close to Bevy's; the
anti-goal is magic (parameter injection, macros), and explicit declarations avoid it.

## Scripts run on one thread

- **One VM on one thread, sequential, with immediate semantics:** change another entity and see it right away. Script
  users never see threads. Within a phase, Rust systems and the script step run one after the other, never at the same
  time.
- A job API for script programmers (each job in its own worker VM, data copied in and out; sequential on the web)
  comes later ([Scripting](../scripting/README.md)).

## Interpolation and the simulated vs displayed pose

- **Rendering interpolates entities moved in the fixed update** between the last two fixed steps, on by default.
  Only entities moved in the fixed update are interpolated, so objects moved in the update render where they are; a
  transform write outside the fixed update snaps that entity (no manual reset); there is a per-entity opt-out. The
  delay of up to one fixed step is documented: twitch games raise the fixed rate or opt out.
  Why: 60 Hz physics would otherwise judder on faster displays, and this scope avoids most of interpolation's usual
  costs.
- **Code always sees the simulated transform,** plus an explicit accessor for the displayed (interpolated) pose, for
  frame-based code that must match the image: a camera following an interpolated character, markers drawn over it, the
  audio listener. Why: interpolated poses depend on frame rate and timing, so gameplay logic reading them would give
  different results on 60 Hz and 144 Hz displays; one transform stays the one truth. Cost: code following an
  interpolated entity must use the accessor or it jitters; the documentation carries that.
- **Interpolation can be built later,** as an addition to render preparation. What it needs is there from the start:
  the retained scene keeps each object's previous model matrix (for motion vectors), the fixed loop knows how far into
  the next step a frame is, and transform propagation marks what moved, tagged by phase. Until it is built, objects
  moved by 60 Hz physics judder slightly on 120 and 144 Hz displays.

## Frame pacing

- **The frame waits before input is delivered,** not inside the frame. Why: winit delivers all input of an event-loop
  iteration before `RedrawRequested`, so a wait inside the frame would push fresh input to the next frame; acquiring
  at the end cut input-to-present latency from about 8 ms to under 0.5 ms in a measurement at identical pacing.
  - **Vsync on:** acquire the next swapchain image at the end of frame N and hold it into frame N+1; the display wait
    happens there. A resize drops the held image before reconfiguring and acquires again at frame start.
  - **Frame cap or vsync off:** the event loop wakes about 2 ms before the deadline (`WaitUntil`) and delivers input,
    then the frame handler sleeps out the rest before the frame runs. Input arriving in that last stretch goes to the
    next frame.
  - **Web:** the browser's animation frame already gives "wait, then events, then frame"; nothing blocks.
- **The frame runs in `RedrawRequested`,** so it keeps working during Windows' modal resize and move loops.
  **Simulation runs once per frame, however many windows there are; each window renders.** A frame isn't a loop
  iteration: on Windows, winit delivers each window's redraw in an iteration of its own.
- **A vsync setting with safe fallbacks:** On (`Fifo`, the default, no tearing), Off (`AutoNoVsync`), Adaptive
  (`FifoRelaxed` where supported, else On), Fast (`Mailbox` where supported, else On). The engine never requests a
  mode the surface lacks. Ignored on the web, where the browser decides.
- **The main window paces the frame:** the vsync setting applies to it, and only its acquire waits for the display.
  Other windows present without waiting (`Mailbox` where supported, else `Immediate`, else `Fifo`): each shows the
  newest frame, with judder on a monitor whose refresh rate differs. While the main window can't present (minimized
  or hidden), the frame limiter paces at its refresh rate. Why: one frame can follow only one display's refresh;
  waiting on every window lets the slowest monitor set the pace, and waits in turn have been reported to divide the
  frame rate. Kept possible: each window rendered at its own monitor's rate ([Later](../later.md)).
- **A frame-rate cap, off by default on desktop,** for vsync off, battery and laptops. It waits at the same point as the
  display wait, as above, without spinning by default (a "precise" option spins the last part). With vsync on, only
  divisors of the refresh rate are allowed (others round down, since they judder). **Each deadline is the previous
  deadline plus the interval,** re-anchored to now when more than one interval late. Why: measuring from the end of
  the frame makes every frame take its work plus a full interval, so the cap always runs slow. For variable refresh
  rate the suggested cap is `r - r²/3600`. On the web, animation frames are skipped by timestamp. Mobile will use the OS
  frame-rate APIs.
- **Background and minimized:** a minimized window doesn't render; an unfocused game runs at a low cap (a setting);
  mobile suspend stops the loop. The editor and every WutEngine GUI application redraw only when something changes:
  input, an explicit `request_redraw()`, an egui repaint request (a delayed one wakes the loop at its time through
  `WaitUntil`), or a woken task ([Jobs and async](jobs.md)). After such an idle wait, the next frame's game delta
  counts as one nominal frame, as the first frame's does (below), so a tool never runs a burst of catch-up steps.
- **Surface robustness:** `Timeout` and `Occluded` from acquire skip the frame, and after an `Occluded` frame a
  reactive application requests another redraw (the first macOS frame may report `Occluded`); `Lost` and `Outdated`
  reconfigure; `Suboptimal` presents that frame, then reconfigures. winit's `pre_present_notify` is called right
  before each present (it paces redraws on Wayland). When no frames come (there is no portable "hidden" signal), a
  timer keeps running logic-only frames at the background rate.
- **Kept possible:** late latching of the camera (recomputing only the view from the newest input right before
  submitting, cheap because per-view data is stored separately); a "low latency" option later (just-in-time frame
  start with one frame in flight).

## Clocks

Plain functions:

- **Game time** `time()`, `delta()`: clamped and scaled, stops while paused. Inside the fixed update, `time()`
  returns `fixed_time()` and `delta()` the fixed step, so `time()` moves step by step there and stays monotonic across
  the frame (the previous update, each fixed step, then this update). Why: gameplay code uses one `time()` and one
  `delta()` everywhere.
- **Real time** `unscaled_time()`, `unscaled_delta()`: never clamped, scaled or paused, so `unscaled_time()` follows
  the wall clock since the frame clock started (UI, menus, the editor, real-time waits). Code integrating
  `unscaled_delta()` must handle large steps: after a breakpoint or a background tab, one frame can span minutes.
- **Fixed time** `fixed_time()`, `fixed_delta()`, plus **`fixed_step_num()`**, the fixed-step counter (for
  multiplayer). `fixed_time()` is the exact sum of the fixed steps actually run: never reset to frame time, and never
  scaled per step (time scale already changed how many steps ran). Not `fixed_step_num() × fixed_delta()`, since the
  fixed rate can change at runtime. Outside the fixed update, `fixed_time()` is the end of the last step run and
  trails `time()` by less than one step (what is left in the accumulator); that difference over `fixed_delta()` is
  the interpolation factor.
- **`frame_num()`.** An audio clock belongs to [Audio](../systems/audio.md).

Representation:

- **Integer nanoseconds inside; absolute times handed out as f64 seconds, deltas as f32;** `_nanos` variants stay
  exact. Why: an f32 absolute time loses precision within hours of play.
- **Shader time is a per-view value that wraps** after a configurable period.

## Time rules

- **The fixed rate is 60 Hz by default,** configurable, also at runtime (for bullet time). Why: it matches the
  displays and most engines; frame-time snapping removes most of the two-steps-or-none jitter on 60 Hz displays.
- **Both deltas come from one measurement, from a monotonic clock at frame start** (`web-time` on the web). The real
  delta is the measured delta; **the game delta is clamped to 0.25 s,** then scaled, and zero while paused. The clamp
  is the only spiral-of-death limit, with no separate step cap: it bounds the fixed steps per frame (15 at 60 Hz), and
  time beyond it is dropped, so an overloaded game slows down rather than running away, and game time falls behind
  real time for good. Why: a hitch, breakpoint or loading pause then doesn't trigger a burst of fixed steps. Kept
  possible: a step cap as a setting if responsiveness under overload turns out to matter.
- **The clamp and the nominal-frame rules (the first frame, after an idle wait) apply to game time only.** Why: they
  protect the fixed-step accumulator, which game time feeds; real time feeds no catch-up steps, and a real-time wait
  must settle after its wall-clock time.
- **The frame clock starts when the event loop does,** so the first frame's game delta is one nominal frame, not the
  time start-up took. Why: otherwise the first frame always hits the clamp and runs a burst of fixed steps.
- **Time scale changes the number of fixed steps, not their length,** and is global only. Why: physics keeps its tuned
  step and a fixed step keeps one meaning, which networking needs; interpolation smooths slow motion. Kept possible:
  a per-scene time scale.
- **Pause is its own flag, not time scale 0.** While paused, game time stops and the fixed update, physics and the
  updates of pausable entities don't run. An entity can be marked **runs while paused** (inherited by its children),
  for menus and pause screens; Rust systems declare it at registration. The development tools' pause is separate and
  freezes all game logic, "runs while paused" included. Why: with time scale 0, scripts keep running with a zero delta,
  so every gameplay script needs its own pause check.
- **Timers and waits** use game time by default, with real-time variants, and don't settle while their entity is
  paused (real time keeps counting for the real-time variant). Their API: [Script API](../scripting/api.md).
- **The web:** browsers pause `requestAnimationFrame` in background tabs; on return the game delta is clamped, and
  real time includes the time away.

## Frame-time measurement

Measuring by presentation is what fixes stutter, but wgpu reports no presentation time yet. Built in this order, each
step replacing one function that produces the frame delta:

1. **The raw measured delta** first. Known cost meanwhile: micro-stutter, and on a 60 Hz display the occasional frame
   with two fixed steps or none; interpolation hides part of it.
2. **Snapping, native with vsync on:** the measured delta snaps to whole multiples of the estimated refresh interval,
   carrying the remainder so game time doesn't drift. Off with vsync off; a frame that clearly missed its refresh
   counts as missed; on a variable-refresh display, where deltas don't cluster, it falls back to the measured delta.
   A setting turns it off. **On the web,** the `requestAnimationFrame` timestamp.
3. **Presentation timestamps** once wgpu exposes them, replacing the estimate where a platform provides them;
   snapping stays the fallback elsewhere.

Why this order: to get going sooner, while measuring in one place keeps each step cheap to swap in.
