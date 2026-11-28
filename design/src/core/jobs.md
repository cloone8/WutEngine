# Jobs and async

How the engine runs work in parallel and waits for things without blocking: the two kinds of work, the task type,
the main-thread executor, and how all of it behaves on the single-threaded web.

## Foundations

- **Async for waiting, threads for computing.** Async: file and network I/O, fetches on the web, GPU readback.
  Threads: asset decoding, render recording, culling, transform propagation, physics. Loading a texture is both:
  reading is async, decoding runs on a worker, and the upload follows ([Assets and project files](../world/assets.md)).
  Audio mixing isn't a job: it runs on the audio output's thread ([Audio](../systems/audio.md)).
- **Engine work is expressed as jobs, written once:** in parallel on desktop, one after another on the web. The main
  thread never blocks waiting. Why: blocking freezes the page on the web, and threads there are an optional later
  accelerator ([Principles and constraints](../goals/principles.md#technical-constraints)).
- **Large batches only.** The scheduler runs small workloads on one thread; the cut-off is measured, not assumed.
  Why: dispatching to a thread pool costs tens of microseconds, so small parallel jobs ran slower than on one thread.
- **Loading happens in the background and never blocks a frame;** results enter the world at frame start
  ([Time and the frame](frame.md)).

## Two kinds of work

Terms: a **job** is work handed to the engine (a frame-work closure or a `compute` closure); a **task** is the result
of later work, which you poll or await.

- **Frame work finishes before the call returns:** a parallel loop over a slice, `join` of two closures, a scope that
  spawns closures borrowing local data. It runs inside a phase (or wherever synchronous code runs), may borrow, and
  returns its results directly; on the web it runs inline, in order. The scheduler, transform propagation, culling,
  physics and render recording use it.
- **Later work finishes after the call returns:** background computation (decoding, building a navmesh) and waiting
  (file reads, fetches, readback). Both give one type, **`Task<T>`**, polled ("done yet? take the result") or awaited
  from async code. Why one type: the caller doesn't care whether it waited or computed.
- **No blocking wait on a task exists in the public API.** Why: on the web it would spin forever, and on desktop it
  would stall the frame. Accepted cost: a Rust user can't load something and use it in a plain synchronous function;
  they await it in async code (start-up included) or check the task each frame.
- **Dropping a task cancels it:** queued work doesn't start, a waiting future is no longer polled, and a computation
  already running finishes with its result discarded. Writing a save is a stated exception
  ([Save games and storage](../world/save_storage.md)).
- **`Task<T>` is `#[must_use]`, with an explicit `detach()`.** Ignoring a returned task is a compiler warning;
  `detach()` lets the work run to completion with nobody waiting. Why: otherwise starting async work without keeping
  its task, the commonest use, would cancel it at once and silently. Script promises aren't cancelled by dropping them.

## Same result whatever the thread count

The engine's half of [parallel equals sequential](frame.md#parallel-equals-sequential):

- **Results by index:** a parallel loop writes each item's result to that item, or returns them in index order, never
  in completion order.
- **Splits don't depend on the thread count:** where the engine combines partial results (a sum, a minimum, bounds),
  chunks are chosen from the data's length only and combined in chunk order, so a float sum is the same on one thread
  and on 32. Why: a thread pool's own reduction order is unspecified, so a float sum could change from run to run.
- **The one-thread path uses the same chunks,** not a plain loop, since chunking changes the last bit of a float sum.
- **Shared state is the caller's responsibility:** a job writing shared state through a lock, an atomic or a channel
  can make order visible again. Engine code doesn't; for user code the rule is documented ("a job writes only its own
  items or returns its result"), with the scheduler's declared access covering systems; there is no runtime check.

## Background work never delays frame work

- A thread waiting for frame work never picks up background work, and background computation runs where it can't hold
  up a phase. Why: long background tasks picked up by waiting threads cause stutter. Accepted cost: on desktop a long
  load finishes later; on machines with few cores "never delays" is best effort.
- What must run on the main thread after background work is spread over frames within the main-thread time slice
  (below).

## Futures run on the main thread, between frames

| Function | Takes | Runs on | For |
|---|---|---|---|
| `run_async(future)` | an async block or `async fn` call | the main thread, between frames, in pieces between awaits | waiting: I/O, fetches, dialogs, readback, multi-step flows |
| `compute(closure)` | a `Send + 'static` closure | a background worker; on the web the main-thread time slice | CPU-heavy work |

Both return a `Task<T>`. Rule for users, as in JavaScript: no long CPU work between two awaits; wrap it in `compute`.

- **One executor for futures on every target,** on the main thread, polled by the event loop between frames, never
  inside a phase. Futures don't need `Send`. Why: a browser promise's future isn't `Send`, so desktop and web code
  stay the same.
- **Desktop file reads** go to a few I/O threads doing plain blocking reads that complete those futures; the web uses
  `fetch`. CPU-heavy steps inside an async flow are handed to `compute` and awaited.
- **On the web the executor yields to the browser:** it is polled from the frame loop or a browser task, never from
  futures that keep waking themselves on the microtask queue.
- **A wake from any thread reaches the event loop,** even when idle, so the executor gets polled; no manual "request a
  frame" after awaiting. Why: wakers are called from I/O threads, libraries' own threads and OS callbacks (a native file
  dialog, for example).
  - **Through winit's `EventLoopProxy::wake_up`,** which wakes the loop. Wakes carry no data, merge, and may arrive
    a few loop iterations late, so the executor keeps its own queue of woken tasks and never counts wakes.
  - **During a headless run's blocking start-up the waker unparks the main thread instead,** since there is no
    event loop.
- **One main-thread time slice between frames** is shared by polling the executor, the follow-up work after
  background jobs and, on the web, `compute` jobs. Why: the three can't add up to more than the frame has.
- **The executor polls once per loop iteration, in winit's `about_to_wait`,** however many windows there are. Every
  iteration ends there, including one started by a wake, so no other callback needs to poll. Windows requested
  since the last iteration are created just before polling, in the same callback, so code awaiting a new window's
  `ready()` resumes one iteration after asking ([Windowing and display](../systems/windowing.md)).
- **Start-up's async part is this executor's first task** on desktop and the web; a headless run drives it with the
  executor's blocking mode ([Lifecycle and memory](lifecycle.md)).
- **No async runtime crate.** Nothing needs one.

## Background computation on the web

All four approaches, each where it applies:

- **Little work by design:** pack data is runtime-ready and large assets come in pieces
  ([Lifecycle and memory](lifecycle.md#memory)), so loading is mostly decompression and copies.
- **Whole jobs on the main thread** within the main-thread time slice. A long user job hitches unless split: the
  documentation says to split long background work, and the engine logs once when a single job overruns the slice,
  naming the job.
- **The browser's own decoders** for standard formats decoded at runtime (images through `createImageBitmap`, with
  colour-space conversion and premultiplication turned off, so PNG gives the same pixels as on desktop; JPEG may differ
  slightly, documented and accepted). A web pack served with HTTP compression is decompressed by the browser.
- **Web Workers for engine job kinds** where they pay off, engine-internal only, built once a measured job kind needs
  them.

Wasm threads stay the later accelerator.

## No blocking inside jobs, no fibers

- **Frame work nests:** a job may open a scope or `join` and help run its children. API shape:
  `jobs::for_each`, `jobs::map_reduce` (fixed chunks), `jobs::join`, `jobs::scope`, with `Send` closures that may
  borrow; a system polls a task with something like `try_take`.
- **A job may not block on a task, a channel or a lock held by another job.** Blocking on a task is impossible by the
  API (no blocking wait; frame-work closures can't `.await`); the rest is documented. Accepted cost: a job can't wait
  mid-way for another; such ordering becomes phases or steps of an async flow.
- **No fibers.** Why: no Rust fiber crate supports wasm, and thread-locals (which memory tags use) are unsafe in a
  fiber that moves between threads.
- **Dependencies between pieces of later work are written as async code awaiting tasks.**

## rayon underneath, as a stand-in

- **rayon does frame work** (scopes, `join`, work-stealing loops, and the sequential fallback on the web), behind the
  engine's own API. Why the own API is needed anyway: rayon has no per-job hook for memory tags, its reductions aren't
  thread-count independent, its `spawn` may never run on the web, and blocking waits are forbidden.
- **Measured defaults for the implementation, not fixed rules:** the main thread is in the pool (much cheaper
  dispatch); the engine checks the batch cut-off before calling rayon; performance cores only by default.
- **Background computation doesn't use rayon's pool.**
- An own scheduler comes later, if rayon's overhead shows up in a real game's profile, or by interest. The engine's API
  keeps that reversible.

## Requirements carried into jobs

- **Memory tags** are captured when a job is queued and entered around its body; per-thread counters are registered
  when a pool thread starts, and by hand for the main thread ([Lifecycle and memory](lifecycle.md#memory)).
- **Profiling:** pool threads are named, and jobs get profiler scopes. **Job names are optional, supported by the API
  from the start;** an unnamed job shows its caller's source location (`#[track_caller]`, no macro). The name or
  location is what the profiler and the time-slice warning show. Why optional: a required name on every parallel loop
  is friction for little gain.
- **Panics abort** ([Error handling](errors.md)).
- **Scripts get no job API now.**

## `Send` in public signatures

**Plain `Send` everywhere,** with wgpu's `fragile-send-sync-non-atomic-wasm` feature on the web. Why: plain `Send` is
what every Rust programmer knows, code that compiles for one target compiles for the other, and the feature is sound
in a build without wasm atomics, which is the web build. If wasm threads come, render code capturing wgpu objects
needs a decision then. Futures need no `Send` on any target.

## Kept possible

A declared dependency-graph API; priorities beyond frame vs background; explicit cancellation tokens; script jobs in
worker VMs; a user-facing Web Worker job API; wasm threads.
