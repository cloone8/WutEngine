# WutEngine

A cross-platform 2D/3D game engine in Rust, written by one developer as a hobby and learning project. The goal is a
Unity-like editor experience, with the engine's internals directly available to Rust programmers. A scripting layer is
planned; no language has been chosen yet.

- **Editor platforms:** Linux, macOS, Windows.
- **Engine and games:** Linux, macOS, Windows, and the web through `wasm32-unknown-unknown` (never emscripten: winit
  and the Rust ecosystem use unknown-unknown). Consoles would be nice later; their SDKs are closed, so don't close that
  door needlessly (no JIT, no hard dependency on a desktop OS in runtime code).

## Design principles

- **Simple for the end user, above all.** Installing and using the engine must be easy: game creators aren't
  necessarily programmers. The user-facing API is simple, with little "magic" (no hidden registration, no behaviour
  that only works because of a macro or naming convention).
- **The editor is the main way to make a game, but not the only one.** A game can be made entirely in Rust, with the
  CLI tools (`tools/`) to create and convert assets. Anything the editor does with assets must stay possible without
  it.
- **GUI tools are WutEngine games.** The editor, and any future tool with a GUI, is built on the engine runtime
  (`wutengine::runtime::start`, egui through `wutengine_egui`), never on eframe or another app framework. Using the
  engine for its own tools is what hardens it: when a tool needs something the engine lacks, add it to the engine.
- **Core engine operations are plain functions.** Loading a mesh, switching scenes, reading config: a free function
  you can call, backed by module-level global state (`InitOnce`, `LazyLock`, `RwLock`). Passing every engine system
  around as an argument was too verbose, so the globals are deliberate. Don't propose context-passing refactors.
- **ECS is for game objects, not for everything.** Entities, components and the systems that run on them use the ECS
  (hecs underneath). Engine services (graphics device, asset server, config, time) are not ECS resources or events.
  The explicit anti-goal is a Bevy-like design where the user API looks simple but the engine code is unreadable.
- **Engine internals stay readable.** Prefer direct code over layers of traits, generics and macros whose only job is to
  make the user-facing API look neat.
- **Stable identity for serialized things.** Components (`Component::ID`) and asset types (`SerializedAsset::ID`) carry
  hardcoded UUIDs that project files reference. Never change an existing one; a new type gets a freshly generated v4
  UUID.

## Workspace

One cargo workspace (`Cargo.toml`), toolchain pinned in `rust-toolchain.toml`.

| Directory | Contents |
|---|---|
| `runtime/` | The engine. `wutengine` is the user-facing crate; the other `wutengine_*` crates are subsystems (graphics, input, physics, audio, time, config, task, event, logger, math, egui integration, development overlay). See `runtime/AGENTS.md`. |
| `asset/` | Asset definitions (`wutengine_assets`), importers, and the asset server. See `asset/AGENTS.md`. |
| `editor/` | The editor (`wutengine_editor`, binary name `wutengine`) and its helper crates `we_*`. See `editor/AGENTS.md`. |
| `tools/` | CLI tools: `weimport`, `weassetconv`, `weshc` (shader compiler), and their shared `wutengine_cli_tools`. See `tools/AGENTS.md`. |
| `util/` | Internal utilities (`wutengine_util`) and proc macros (`wutengine_util_macro`). Not for use outside the workspace. |
| `build/webuild` | `cargo webuild`: release-builds the tools and editor and copies them into `dist/` (gitignored). |
| `docs/` | An mdBook manual, currently only an introduction. |

Each crate's `README.md` is included as its crate docs (`#![doc = include_str!("../README.md")]`), so editing a README
changes rustdoc.

## Working with the user

- **Push back and suggest.** The user's request is a starting point, not the law. Make active suggestions on code and
  engine architecture. If a request or an existing design looks like a bad long-term decision (it conflicts with the
  design principles, blocks a target platform, adds magic to the user API, or paints the engine into a corner), say
  so plainly, explain why, and offer concrete alternatives with their trade-offs. The user then decides. Raise it
  before doing the work, not after; once the user has decided, build what they chose without re-arguing.
- A question like "what's the next step?" asks for an explanation: what, why, how, risks, and any decision the user
  has to make. Answer and stop; wait for an explicit go-ahead before doing it. Read-only lookups to make the answer
  accurate are fine.
- When the user makes a remark meant for future sessions, walks you through a procedure, or you repeat a workaround
  from an earlier session, end your reply with a one-line suggestion to capture it in the repo. The
  `capturing-instructions` skill says where it belongs. Add nothing until the user agrees.
- When a skill, hook, `AGENTS.md` line or script in this repo turns out wrong or incomplete, say so. Fix a plain
  factual error (a path, a flag, a name) in the same change and mention it; for anything that changes what an
  instruction asks for, including any hook change, suggest the edit in one line and wait for a yes.
- When a tool a skill or hook needs is missing, tell the user which tool, what needed it, and how to install it. Never
  skip the step silently. The session-start hook lists missing tools.
- Never add a `CLAUDE.md` anywhere in this repository: Claude Code stops reading `AGENTS.md` files as soon as one
  exists.

## Git

The main branch is `master`. **The user commits and pushes; agents don't** (`.claude/settings.json` denies it). Leave
changes uncommitted and unstaged. When the work mixes unrelated changes (e.g. a mechanical reformat and a feature), list
which files belong to which so the user can commit them separately. If you have to stash, use a tagged stash
(`git stash push -u -m "<unique-tag>"`, then apply by that entry's SHA); the stash stack is shared by every worktree
and session, so never a bare `git stash`/`pop`.

## Subagents

The built-in `Explore` and `Plan` agents may not load these instructions. Use `Explore` only to find code; for
designing or reviewing a change use a general-purpose agent or `branch-reviewer`. No subagent sees the conversation:
brief it on the user's corrections and decisions that apply to its task.

## Notes and state documents

Working notes go in `.claude/notes/`, which is gitignored: investigations, measurements, and the progress and state of
ongoing work (what's done, what's unverified, decisions made along the way). Reference them by full path
(`.claude/notes/<name>.md`). Read the notes there at the start of a task and keep the ones you rely on current. The
committed files (`AGENTS.md`, skills, `docs/`) hold only what stays true; never put dated "current state" or progress
notes in them. `docs/` is the user-facing manual, not a scratch space.

State documents (`PENDING.md`, a plan or handoff the user asks for) go where the user says, usually the worktree root,
not gitignored and never committed. Keep them current without asking. For any handoff request, use the `handoff`
skill.

## Investigations

Get the documentation of whatever you work with and treat it as the source of truth: docs.rs for the exact versions
in `Cargo.lock` (wgpu, naga, winit, hecs, rapier, egui, ...), and the WGSL and WebGPU specs. Don't reason about an
API's behaviour from memory. Documentation in this repository may be stale; fix it in passing when you find it wrong.

## Programming style

This applies to all code you write, and to every file you touch. The best code is the code never written.

Stop at the first rung of the ladder that holds:

1. **Does this need to exist at all?** Speculative need: skip it and say so in one line.
2. **Already in this codebase?** Reuse the helper, type or pattern that lives here (`wutengine_util` has `InitOnce`,
   `MainThreadOnly`, `JobQueue`, `map!`, `log_once!`, ...). Look before you write.
3. **Does std do it?** Use it.
4. **Does a dependency already in the workspace do it?** Use it. Never add a new one for what a few lines can do.
5. **Can it be one line?** One line.
6. **Only then:** the minimum code that works.

The ladder runs *after* you understand the problem, not instead of it. Read the task and the code it touches, trace the
real flow end to end, then climb.

**A bug fix fixes the root cause.** Before editing a function, find every caller. One fix in the shared function beats
a guard in each caller.

- No unrequested abstractions: no trait with one implementation, no generic with one instantiation (a const-generic
  switch is fine), no config for a value that never changes.
- No scaffolding "for later". Deletion over addition. Boring over clever.
- Fewest files, shortest working diff, but only once you understand the problem.
- Complex request? Ship the simple version and question it in the same reply: "Did X; Y covers it. Need full X? Say so."
- Mark a deliberate simplification with a known ceiling with a `TODO:` naming the ceiling and the upgrade path.
- Never simplify away: input validation at trust boundaries (asset files, shader sources, user input), error handling
  that prevents data loss, or anything the user asked for explicitly.

**Output:** code first, then at most three short lines on what was skipped and when to add it. No unrequested essays.
Explanations the user asked for (a report, a walkthrough) are given in full.

## Rust

- Before reporting Rust changes as done (also as a subagent), run the **`rust-review`** skill over your diff and fix
  what it finds.
- `cargo clippy` must report no warnings in code you touched. The workspace lints (`[workspace.lints]` in
  `Cargo.toml`, plus `clippy.toml`) are strict: clippy `pedantic`, docs on every item including private ones, `mod.rs`
  module files only, and a `reason = "..."` on every lint suppression (prefer `#[expect(...)]` over `#[allow(...)]`).
  Every crate opts in with `[lints] workspace = true`.
- Format with `cargo +nightly fmt`, never plain `cargo fmt`: `rustfmt.toml` uses unstable options that stable rustfmt
  skips with only a warning. In particular one `use` per item (`use a::B;` and `use a::C;`, never `use a::{B, C};`). A
  PostToolUse hook formats every `.rs` file you edit or write; run the command yourself for files changed another way.
- Dependencies are declared in the root `Cargo.toml` with `default-features = false` and used as
  `dep.workspace = true`, enabling features per crate. Don't add a crate from crates.io without asking.
- Errors are `derive_more` enums (`Display`, `Error`, `From`), never `thiserror`, `anyhow` or `String`. Use the
  `log` macros for logging and `profiling::function_scope!()` / `profiling::scope!()` for profiling, like the
  surrounding code.
- Use the type system aggressively, and prefer a compile-time guarantee over a convention callers have to remember:
  newtypes and enums instead of strings, integers and `bool`/`Option` combinations; invariants in types (a guard or
  token the protected functions are methods on, private constructors); standard traits (`From`/`TryFrom`, `Default`,
  `Display`/`FromStr`, `AsRef`, `Iterator`) instead of one-off conversion functions. Where this and the ladder pull
  apart, this wins.
- Comments give the reason the code can't show, in one or two sentences. Doc comments on private items are one line.
  How the code came to be belongs in the commit message. If code needs many comments, refactor it.
- Files use LF line endings.

## Cross-platform code

- Avoid platform-specific code: prefer std, cross-platform crates already in use, and the engine's own abstractions.
- Where it's unavoidable, give every target its own implementation, selected with `cfg_select!` (as in
  `runtime/wutengine_task/src/detect.rs`), not a stack of `#[cfg]` items and never only a Windows or Linux path.
- Runtime crates also run on the web: see `runtime/AGENTS.md`. The editor, tools and `build/` may assume a desktop OS.

## Tests

Add unit tests freely, in any crate, wherever one can verify the logic you changed. Keep them to pure logic: the engine
can only be started once per process and needs a window and GPU, so there is no harness for running it in tests.
Every test gets a one- or two-line doc comment saying why it's useful: "Tests X to prevent Y, which happens when Z".
Run them with `cargo test`.

## Commands

| What | Command |
|---|---|
| Check everything | `cargo clippy --workspace --all-targets` |
| Format | `cargo +nightly fmt --all` (check: add `--check`) |
| Test | `cargo test --workspace` |
| Build the tools and editor into `dist/` | `cargo webuild` |
| Run the editor | `cargo run -p wutengine_editor -- [path/to/project.we-project]` |

Build, check, test and run the CLI tools freely. Ask before launching the editor or anything else that opens a window.

Do asset and shader work through the repository's own tools (`tools/AGENTS.md`), the way a game creator would: compile
and inspect shaders with `weshc`, import with `weimport`, convert with `weassetconv`. Don't reach for the underlying
library directly (naga, an image crate, a hand-written script) when a tool covers the job; running the tool is also
what finds its bugs. When the pipeline has no tool for a step you need, say so and suggest the tool (name, input,
output) instead of working around the gap.
There is no CI: these commands are the only checks.
