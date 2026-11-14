# Building and shipping

How a project becomes a game: compiling the player, what a machine needs, asset packs, build settings, the web output
and the players. Anything the editor does here is also possible from the command line.

## Every project compiles its player

**A game is engine code plus the project's Rust code (if any), scripts and assets, and every project compiles its own
player with cargo.** The editor ships prebuilt, detects rustup and the per-OS prerequisites, and guides the install of
what's missing; the project's `rust-toolchain.toml` then makes rustup install the pinned toolchain on the first `cargo`
call. The editor and the build CLI invoke cargo. This also covers a designer in a team: their editor compiles the
project's player too.
Why: Rust code from a programmer reaches every teammate. Accepted: a one-time toolchain install per machine, also
for non-programmers in a team, and a first build that takes minutes with a large `target` folder.

What a machine needs besides rustup:
- **Windows:** the MSVC toolchain's linker and Windows SDK (which rustup-init offers to install through Visual Studio,
  whose free licence doesn't cover every organisation).
- **macOS:** the Xcode Command Line Tools.
- **Linux:** a C toolchain plus the ALSA development files, for audio output ([Audio](../systems/audio.md)).

**Cross-OS builds:** native and web builds work on the developer's own OS (web from any host); builds for another OS
happen on that OS or on CI.

**Planned, not a priority: prebuilt players for script-only projects** (no toolchain, no first-build wait, builds for
any OS from any OS). They need per-platform player builds for every engine release, so CI with a macOS runner. Additive;
nothing above closes it off.

## Asset packaging

- **Generic asset pack files whose grouping is opaque to the player.** The player only knows "asset X is in pack Y at
  offset Z" (with size and compression), from one small **manifest** written at build time and read first; it also
  holds each asset's dependency list ([Assets and project files](../world/assets.md)). The build pipeline decides the
  grouping per target platform; on the web the manifest tells the player which pack to fetch.
- **One pack format everywhere:** a header, an index from asset UUID to offset, size and compression, then the data.
  Mounting several packs is in the format from the start. Compression is per asset inside the pack, to keep random
  access.
- **First grouping:** desktop gets one pack; the web gets one pack per texture-format variant plus one for everything
  else (the whole game downloaded at start). Later: automatic grouping by scene for the web (assets used by one scene
  in its pack, shared assets in a shared pack), each pack under 200 MB.
- **One async "bytes for asset ID" interface** with three sources: the project's import cache (play mode, no
  packing), pack files (desktop builds), packs fetched over HTTP (the web) ([Assets and project
  files](../world/assets.md)).
- **Texture variants:** the player picks a variant by GPU support; whether the manifest maps an asset to one location
  per variant or the build writes one manifest per variant is [open](../open.md) ([Shaders and
  materials](../rendering/shaders.md)).
- **Kept possible: patches and DLC,** where a later manifest's entries override earlier ones.
- **Build contents:** only what's referenced (from the scenes in the build, through typed asset references in
  components and exported script properties), plus assets or folders explicitly marked **"always include"** for assets
  chosen at runtime. Why: exporting everything bloats every build; the mark is explicit and visible.
- **Loading screens on the web:** the manifest answers "which packs do these assets need", so a scene can be preloaded
  with progress.
- **Inspecting builds:** a CLI tool lists a pack's contents and finds an asset.
- Why packs rather than one file per asset: async loading needs random access to any one asset; many small files are
  slow to open on desktop, and web hosts limit file counts (itch.io: 1,000 files). Known cost: changing one asset
  rewrites its pack; play mode reads the import cache, so only real builds pay.

## Build settings and outputs

- **Scenes in a build:** an explicit list in the project's build settings, plus an explicitly chosen **start scene**
  (not "the first in the list"). Reference tracking starts from these scenes plus the "always include" marks
  ([Scenes](../world/scenes.md)).
- **Build CLI: `webuild`,** one tool that builds a project for a target; the editor's Build button calls it. The
  engine repository's own internal tasks use `cargo xtask` (such as `cargo xtask dist`), so the names don't clash.
- **macOS:** an `.app` bundle, built on a Mac, as a universal binary (arm64 and x86_64; arm64 is the main target).
  Unsigned by default, with a note for players on how to open it. Signing and notarization later, opt-in, with the
  developer's own Apple account.

## Web output

**A folder plus a zip ready for itch.io:** `index.html`, the `.wasm`, wasm-bindgen's JavaScript glue, the manifest and
the packs. The build installs the `wasm-bindgen` CLI version that matches the crate in `Cargo.lock`, and checks
itch.io's limits (1,000 files, 200 MB per file, 500 MB total), failing with a clear message.

- **The loader library is the engine's whole promise to a web page:** one JavaScript function that takes the canvas
  to render on, the command-line arguments ([Configuration](../core/configuration.md)) and options, at least a
  progress callback and a fatal-error callback, so any page can show a loading bar and the crash message.
- **The page comes from a web template, in every build:** built-in templates are thin pages over the loader, and a
  project can add its own. The default template checks for WebGPU before downloading the wasm and otherwise explains
  what's missing (and can link a desktop download), shows a progress bar while packs download, and shows the crash
  message. A blank "do it yourself" template is a canvas and one call to the loader. Why in every build: every web
  release would otherwise need a hand-written page, against a folder ready for itch.io.
- **Audio:** the loader also loads the mixer's small wasm module into an audio worklet, resumes audio on the first
  click, tap or key press, and suspends it while the tab is hidden; the default template has a "click to start"
  option, off by default ([Audio](../systems/audio.md)).
- **A build option for no page:** the loader and the game without an `index.html`, for developers who embed the game
  in their own site.

## Players

- **Two players at first:** the **play-mode player** (editor link, development overlay, debug assertions) and the
  **release player** (none of those). A "development build" (release speed plus logging and the overlay) later.
- **Play mode compiles dependencies optimized and the project's own code unoptimized.** Why: heavy dependencies run
  fast while the project still compiles quickly.
- **From the error policy** ([Error handling](../core/errors.md)): `panic = "abort"` in every profile; plain `assert!`
  stays in release builds; the release player always writes a log file and keeps the previous run's; a crash shows a
  native message box (desktop) or a page message (web).
