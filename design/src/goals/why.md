# Why the engine exists

What WutEngine is for, who uses it, how goals are weighed against each other, and what the engine builds itself.

## Purpose

> WutEngine is my engine for making my own games. I build it because it's fun, and to learn graphics, game engine
> development and tooling. It's built so that anyone can pick it up: at a game jam, teammates who never saw it,
> programmers or not, can be productive within an afternoon through a graphical editor that programmers extend. I
> don't need it to be widely adopted.

- **Success test:** I can make my games with it, and it is jam-ready (below).
- **Rust is a personal choice** (fun, tooling, safety), not a bet that has to beat other Rust or open-source engines.
- **Monetization is not a goal.** If it ever becomes one, console ports (closed NDA code anyway) fit an open core.

## Who makes games with it

Everyone: solo developers to teams, programmers and non-programmers, working with the same tooling.

- **A graphical editor is a must.** Why: non-programmers make games through it.
- **Programmers extend it:** they can change the core of their game and of the editor experience, and expose new
  functionality to non-programmers (the Unreal C++ to Blueprint model). Why: one team, two kinds of member.

## Priorities when goals conflict

Within the [technical constraints](principles.md#technical-constraints), roughly in this order:

1. My game and my learning.
2. Ready for others: jam-ready, editor first, simple API, extensible by programmers, docs and tooling.
3. Seeking adoption: community, marketing, contributor management. Optional.

## Jam-ready

Jam-ready is the benchmark for "ready for others".

- **Install:** a prebuilt editor download. The user installs rustup and the per-OS prerequisites; the editor detects
  what's missing and guides the install, and rustup installs the project's pinned toolchain
  ([Building and shipping](../editor/shipping.md)).
- **Getting started:** a getting-started guide and a template project; the template includes a drop-in first-person
  player scene ([Physics](../systems/physics.md)).
- **Web export** (`wasm32-unknown-unknown`) is a first-class target from the start. Why: jam entries are mostly played
  in the browser.
- **Teams in version control:** text project files and one settings file per asset, so adding assets doesn't conflict
  ([Assets and project files](../world/assets.md)).
- **Iteration fast enough for a 48-hour jam.**
- **No data loss when the editor crashes.** Play mode runs in its own process, so a crashing game doesn't take the
  editor down ([Editor](../editor/README.md)). Accepted: a buggy editor extension (project Rust code or TypeScript
  edit-mode code, which run inside the editor) can crash the editor and lose unsaved work. Crash recovery by a change
  journal isn't needed early, but is required before the engine is called jam-ready.

## The first game

A small 3D project with a very simple PSX-style look, as many indie horror games use now: unlit is fine, maybe some
post-processing. The engine still targets both 2D and 3D.

What the look needs from the engine, mostly general features rather than special cases
([Shaders and materials](../rendering/shaders.md)):

- a low-resolution render target upscaled with nearest filtering (a view rendering into a texture, then a fullscreen
  pass);
- vertex snapping (the vertex stage adjusts the clip-space position to the low-resolution pixel grid);
- affine texture warping (a custom interpolator with WGSL `@interpolate(linear)`);
- nearest texture filtering, low-resolution textures, vertex colours for baked lighting;
- distance fog (to black);
- post-processing: colour quantization with dithering, optional noise, vignette and CRT effects.

The look itself is a user-defined shading model ("PSX", on the surface domain) plus fullscreen post materials. Why: it
dogfoods user shading models and injection points.

## Build it yourself by default

Everything that is engine or game programming is built for WutEngine by default: the object model, audio, animation,
in-game UI and text, navigation, networking, asset import beyond image decoding. Crates for engine parts can still be
chosen case by case. General infrastructure crates stay (`log`, `serde`, `uuid`, `winit`; graphics through `wgpu`).
Why: learning game engine development is the engine's first purpose.

Exceptions and crates in use:

| Crate | Status |
|---|---|
| `winit` (windows and events) | Kept, on 0.31 while it is a beta, pinned to an exact beta. egui gets the engine's own winit integration, since egui-winit still requires 0.30 |
| `glam` (math), `image` (image decoding), `naga` (shader translation), `egui` (editor and overlay UI), `puffin` (profiler) | Kept |
| `hecs` (the ECS under the object model) | Kept, behind WutEngine's own API ([Object model](../world/object_model.md)) |
| `rapier` (`rapier3d`, `rapier3d-f64`, later `rapier2d`) | Kept, assumed permanent, behind the engine's own physics API ([Physics](../systems/physics.md)) |
| `harfrust` (text shaping) | Kept ([In-game UI and text](../systems/ui_text.md)) |
| `parley` (text layout), `taffy` (layout) | Stand-ins |
| `rayon` (frame work under the job system) | Stand-in ([Jobs and async](../core/jobs.md)) |
| `cpal` (desktop audio output), audio decoder crates | Stand-ins; the mixer is the engine's own ([Audio](../systems/audio.md)) |
| `gilrs` (gamepads) | Stand-in behind the engine's own gamepad API; Android needs another backend later ([Input and camera](../systems/input_camera.md)) |
| A node editor crate | Not used: the node editor widget is built on egui |

- **The ECS is the exception to building it yourself:** an existing crate wherever one fits, hecs, wrapped in the
  engine's own API; an own ECS only if no crate fits. Why hecs: deterministic query order on every target, safe
  parallel queries, and no scheduler, resources or events of its own ([Object model](../world/object_model.md)).
- **Physics is rapier, assumed permanent.** Why: writing own physics isn't expected; the engine's own API stays in
  front of it.
- **Stand-ins may ship.** The first game may ship on stand-in crates where own versions aren't ready. Replacing a
  stand-in is learning work, scheduled by interest; it gates neither the first game nor 1.0. Why: the game comes
  first, and a stand-in behind the engine's own API can be swapped without changing user code.
