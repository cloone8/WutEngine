# Assets and project files

How project files are stored and survive engine updates and teams, and how the running game references, loads, frees,
validates and reloads assets.

## Project files

- **Committed project files are self-describing text:** scenes, materials, settings, per-asset import settings.
  Pretty-printed with stable key order, so diffs and merges stay small. Component data is written inline as values
  (reflection's serde value trees), never as opaque bytes. The text format is **JSON**; configuration files are TOML
  ([Configuration](../core/configuration.md)). Why: binary formats without field names can't evolve (an added field
  fails to load, reordered fields load silently wrong), and JSON is known to almost everyone and works in version
  control. Any replacement format must store field names too. Known cost: no trailing commas, so appending to a list
  also changes the previous line.
- **Regenerable data may be binary:** the import cache and build packs, both rebuilt by the current engine version, so
  evolution doesn't matter there and the binary format is replaceable.
- **Source files plus one small settings file per asset** (its UUID and import settings, next to the asset like Unity's
  `.meta`) are committed; the import cache isn't, and is rebuilt when the engine version changes. Bulk data lives only
  in source files. Why: committed imported binaries are unmergeable, need a locking workflow, can drift from their
  source, and must themselves evolve across engine versions; with this model, importer improvements also reach every
  project. Kept possible: a shared import cache, if import times grow.
- **Lazy conversion in play mode** is a goal: play mode converts only what it loads, asking the editor (or a conversion
  worker) for an asset, which imports it first if it's missing or stale. Builds convert everything they include.
- **Every entity has a stable UUID in scene files,** so references between entities survive saving and merging
  ([Object model](object_model.md)).
- **Compatibility** follows [Version 0 stays flexible](../goals/principles.md#version-0-stays-flexible).

## References: `Asset<T>` and `AssetRef<T>`

- **An `Asset<T>` always refers to usable data:** `get()` returns the asset, never "not loaded yet". There is no load
  state, so no code checks one. Residency within an asset (which mip levels are in memory) is the asset type's own
  business, which keeps streaming within an asset possible. Why a type rather than a plain shared pointer: it
  remembers its asset ID (for files and the editor), survives hot reload, is freed at a fixed point and feeds the memory
  counters. Code that wants an entity to appear before its mesh spawns it and sets the mesh when the load completes.
- **Loading returns a `Task`** ([Jobs and async](../core/jobs.md)) that completes with the `Asset<T>` once the asset and
  everything it hard-references are loaded, or with the error. A dynamic load's error therefore arrives as a `Result`
  (Rust) or a rejected promise (scripts), once.
- **Loading the same asset twice gives the same data:** one copy per asset.
- **Two typed field kinds** in components and exported script properties: **`Asset<T>` (hard)**, loaded with the scene
  before it enters the world, and **`AssetRef<T>` (soft)**, a typed asset ID loaded only when code asks. The editor
  shows both as the same asset picker; the type decides when it loads. Hard is the default; soft is for large or rarely
  used assets (a death effect loaded when the enemy dies).
- **Inside a scene, hard references are always loaded:** a script on a scene object never sees a missing hard-referenced
  asset. Why: that is the non-programmer path, and it can't fail at runtime in a correct build.
- **Both are in the build:** the build follows hard and soft references alike, through reflection; only hard ones load
  with the scene.
- **Hard-reference cycles are a build error** (and an editor warning when the field is set); soft references may form
  cycles. The error names the chain of assets and fields and the fix (change one field to an `AssetRef`). Why: in a
  hard cycle each asset waits for the other to load and keeps the other's count above zero, so it never finishes
  loading and is never freed. Loading and freeing a hard cycle as one group comes later, so a scene that spawns
  instances of itself just works.

## Dynamic loads

- **By soft reference** (the normal case), **by asset ID**, or **by path**.
- **Loading by path works only for assets marked "always include",** in play mode as in builds; the build ships a path
  table for those assets alone. Loading by ID works for anything in the build. Why: identity is UUIDs so that renames
  don't break references; a path table for everything would let renames silently break string paths in code. Cost
  accepted: an asset must be marked before it can be loaded by path.
- **Play mode applies the build's rule:** a load the build would refuse fails in play mode with the same error, naming
  what to mark. Why: otherwise the bug would appear only in builds.

## Lifetime

- **Assets are counted;** when the last reference drops, the asset is freed at the end of that frame, never mid-frame.
- **No grace period by default.** Why: freeing at once is what makes asset counts return to their baseline after a
  scene unload ([Lifecycle and memory](../core/lifecycle.md#memory)). Churn at a scene switch is avoided by order
  instead: the next scene's references are counted before the old scene's are released. Kept possible: keeping a set of
  assets loaded on purpose.
- **Assets held by scripts are freed by the script garbage collector:** a script's asset object holds a count like any
  `Asset<T>`; when the collector frees it, the count drops and the asset is freed at the end of that frame. **The
  engine forces a script collection at a scene `switch`,** and only there. No `release()`. Why: tying script-held assets
  to entities or explicit releases was complicated and bug-prone; forcing a collection at a switch, a loading moment
  anyway, makes the baseline check there deterministic; elsewhere a collection costs a few milliseconds that open worlds
  loading scenes constantly can't afford. Between scene changes an asset a script stopped using may stay loaded until
  the collector runs; it is never freed while in use.

## The load pipeline

- **Dependencies are in the manifest:** the build writes each asset's hard references into the manifest, which is read
  first and already answers which packs a load needs ([Building and shipping](../editor/shipping.md)). A load knows its
  whole graph and every pack it touches before reading any asset, and issues the reads (on the web, the pack fetches)
  together. Why: with dependencies in each pack's index, the web loader would have to fetch one pack to learn the next.
- **Steps:** read bytes (desktop I/O threads, `fetch` on the web, under the bytes-in-flight cap) → decompress and
  decode (`compute`; on the web the [web rules](../core/jobs.md#background-computation-on-the-web)) → upload →
  ready.
  A scene enters the world at frame start once its hard references are ready. Memory rules for loading:
  [Lifecycle and memory](../core/lifecycle.md#memory).
- **GPU uploads on native are the last step of the decode job,** not main-thread work: the job writes into staging
  memory and schedules the copy, which wgpu runs at the frame's submit; the main thread only submits and marks the asset
  ready at frame start. Few large writes rather than many small ones. Why: wgpu's queue writes are thread-safe and copy
  at the call, so only the submit needs the main thread. A bytes-per-frame upload cap bounds staging memory and GPU
  copy work. Textures are not uploaded with `write_texture` from jobs, since in wgpu it holds the lock the submit
  needs: the job writes texels into a staging buffer and the frame records a buffer-to-texture copy, or the upload
  goes in small pieces. Buffers use `write_buffer`, per-frame data a staging ring, and there are no `unsafe` direct
  staging writes.
- **On the web, uploads stay on the main thread** within the main-thread time slice, and images the browser decoded are
  copied straight to textures.
- **Pack data's row layout for texture mips is part of "runtime-ready"** (rows padded to wgpu's copy alignment).
- **Dropping a load task cancels the parts no other load needs.** Kept possible: priorities beyond "needed for a scene"
  and "prefetch".

## Failures, placeholders and integrity

- **Placeholders per asset type, built in:** a magenta texture and error material, a visible placeholder mesh, silence
  for audio, an empty value for data assets. Play mode uses them for a missing or failed scene reference, logging the
  scene, object and asset ([Error handling](../core/errors.md)).
- **Pack data is a trust boundary:** every decoder validates and returns an error, never panics or asserts.
- **A hash per asset in the pack index,** checked when the asset is read. A **hash mismatch** means a damaged install
  ("game files are damaged"); a **validation failure on bytes whose hash matches** is an engine bug, logged with the
  asset's ID and type. Why: it tells a damaged install from an engine bug. No start-up scan of whole packs; the check
  may be skipped where it isn't needed (play mode reading the import cache).

## Hot reload

The API shape hot reload needs is there from the start (assets whose data can be swapped, a changed event, per-type
"replace my data"), so adding reload later doesn't change user code; which types reload, and when, is decided when it
is built.

- **Over the editor link:** the editor reimports a changed source and tells play mode, which reloads the asset from the
  import cache; no file watcher in the player. Development only.
- **Same `Asset<T>`, new data, swapped at frame start:** every holder sees the new version; engine types redo their own
  derived work. A **changed event** tells code that copied data out of an asset; for scripts it is a signal on an
  engine service object, delivered at the settle point ([Signals](../scripting/signals.md)).
- **A failed reload keeps the old data** and logs the error. A reloaded scene applies to later loads and spawns
  ([Scenes](scenes.md)).

## Assets created at runtime

- **Code can create assets from data** (a procedural mesh or texture, data read at runtime). They get the same
  `Asset<T>`, counted the same way, with no asset ID; saving a scene that holds one in a field is an error naming the
  field.
- **One format library, shared by the editor, the CLI tools and game code,** layered: **readers** per format (image,
  OBJ, glTF, FBX, ...) and **converters** on top (GPU texture compression, mips, mesh optimization, axis and unit
  conversion), mostly used at import time because they are slow. Readers produce the engine's own data types, the same
  ones "create an asset from data" takes, so at runtime the chain is read, create, `Asset<T>`.
- **Each format is a cargo feature,** so a game pays only for what it enables (wasm size above all).
- **Images on the web go through the browser's decoder** behind the same function; the engine's own image reader stays
  available through its feature for a game that needs identical pixels everywhere.
- **Readers validate their input,** more so at runtime, where a downloaded file is untrusted.
- **Runtime availability is decided per format and per target:** a reader wrapping a C or C++ library (FBX, USD) may
  exist on desktop and stay import-only on the web.

## References in serialized data

1. **In files every reference is a UUID:** an `Asset<T>` field, an `AssetRef<T>` field and an entity reference alike.
   The type decides only when an asset loads, so changing a field's kind changes no files.
2. **The schema marks reference fields** (hard asset, soft asset, entity). One rule serves the build (what to include),
   the editor (asset or entity picker), the scene loader and the generated `.d.ts`.
3. **Load before deserializing:** a scene's hard references are known before it is read (from the manifest in builds,
   from the scene's import in play mode); the scene load loads them first, then deserializes.
4. **Deserializing can't fail on references:** an `Asset<T>` field looks its UUID up among the assets the load holds;
   entity references resolve in a second pass, once every entity in the file exists.
5. **Saving is the reverse:** an asset writes its UUID, an entity reference the entity's UUID. Overrides on scene
   instances build on this ([Scenes](scenes.md)).

Deserializing a component outside a scene load goes through the same "load references first" step. Save files don't
need it: they hold game data, not components ([Save games and storage](save_storage.md)).

Package assets keep their UUIDs ([Packages](packages.md)). The asset types themselves and the editor's asset database
come later.
