# Multiplayer

Multiplayer's ambition, its model, and what the engine keeps possible now so networking can be added later without
redesign. Networking itself comes later.

## Ambition and model

- **A goal someday.** The stretch goal is large dedicated-server games, not MMO scale.
- **Server-authoritative:** servers decide the game state; clients send input and receive state. Why: replication
  doesn't need identical simulation across platforms, which the engine doesn't promise.
- **No promise of identical simulation across platforms,** so lockstep and rollback netcode are not designed for.
  Why: floating point differs between x86, ARM and wasm.

## Kept possible now

1. **A headless runtime** (no window, no GPU) for dedicated servers, also useful for automated tests; graphics,
   windowing and audio are optional, not assumed ([Lifecycle and memory](../core/lifecycle.md)).
2. **A fixed-step number** on the fixed-step loop ([Time and the frame](../core/frame.md)).
3. **Input as per-step data, separate from the device that produced it,** so it can be sent or replayed: a per-step
   input record per local player ([Input and camera](../systems/input_camera.md)).
4. **Network identity for entities spawned at runtime** (scene entities already have UUIDs;
   [Scenes](../world/scenes.md) keeps derived UUIDs for spawned entities possible).
5. **Replicable state described by reflection:** components and script properties
   ([Reflection](../world/reflection.md)).
6. **A transport abstraction.** Why: browsers have no raw UDP, so the web goes through WebSockets, WebRTC data
   channels or WebTransport.
7. **A seeded random-number API:** a global `random()` family for convenience (seeded from the clock, or from a
   configuration setting for reproducible runs), plus explicit generator objects with their own seed for anything that
   must replay identically (procedural generation, networking, replays), as Unity's `Random` next to `System.Random`
   instances.
8. **Editor play mode with several players** (a server and clients), which fits play mode as a separate process
   ([Editor](../editor/README.md)).

Already helping: fixed-step simulation, stable scene UUIDs, deterministic order within a build
([Jobs and async](../core/jobs.md)), absolute positions on the network ([Game size and precision](game_size.md)).

## Consequences of the stretch goal

- **Heavy server-side logic belongs in Rust.** Why: scripts run on one thread (roughly 340–380 moving objects per
  millisecond natively), while Rust systems run in parallel. Script jobs in worker VMs come later
  ([Scripting](../scripting/README.md)).
- **Dedicated servers build with f64 world positions** and hold the whole world, while clients stay f32 with origin
  shifting; the network carries absolute positions ([Game size and precision](game_size.md#precision-build-switch)).
