---
name: rust-review
description: Use when about to report Rust changes as done, before handing a Rust diff back to the user or to a parent agent, or when asked to review Rust code in this repository for its conventions.
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
---

# Rust review

A pass over your own diff before you hand it back. Get the diff (`git diff master -- '*.rs' '*Cargo.toml'` plus
untracked files), walk every changed item against the table, and fix what matches. Then run
`cargo clippy --workspace --all-targets` (no warnings in touched code) and `cargo +nightly fmt --all`.

## Checklist

| Look for | Write instead |
|---|---|
| A custom conversion or constructor function: `fn texture_from_image(img) -> Result<Texture>` | A standard trait: `impl TryFrom<Image> for Texture`. Also `From`, `Default`, `Display`/`FromStr`, `AsRef`, `Iterator` |
| Strings or integers that stand for a fixed set of values | An enum, with `Display`/`FromStr` if it crosses a text boundary |
| A `bool` parameter every caller passes as a literal | A const generic, as `InitOnce<T, UNCHECKED, MAIN_THREAD_ONLY>` does |
| An invariant callers must remember | Encode it in a type: a guard or token the functions are methods on, a private constructor |
| Hand-written `Display`/`From`/`Error`/`is_*` impls, `thiserror`, `anyhow`, errors as `String` | `derive_more` (`Display`, `Error`, `From`, `IsVariant`, `Unwrap`; enable the matching feature in the crate's `Cargo.toml`). A derive named like a std one is path-prefixed: `#[derive(derive_more::Debug)]` |
| A stack of `#[cfg(...)]` items selecting per platform | `cfg_select! { target_os = "windows" => { .. }, _ => { .. } }`, as in `wutengine_task/src/detect.rs`, with a branch for every target |
| In a runtime crate: `std::fs`, `std::thread`, `Instant`/`SystemTime`, `block_on`, sleeping, OS APIs | A web-compatible path, or keep it out of the runtime crate (`runtime/AGENTS.md`, The web target) |
| A new engine feature as an ECS resource, event or system, or a context parameter threaded through calls | A plain free function over the module's global state (root `AGENTS.md`, Design principles) |
| A global read from code that may run before its `init`, or main-thread-only state touched from a system | Initialize it earlier, or go through `send_to_main_thread` (`runtime/AGENTS.md`, Global state) |
| A changed `const ID` on a component or asset type, a renamed or removed serialized field | Keep the old ID; `#[serde(alias = "old")]` for renames (`asset/AGENTS.md`, Compatibility) |
| `#[allow(...)]`, or any lint suppression without `reason = "..."` | `#[expect(lint, reason = "...")]` |
| Grouped imports `use a::{B, C};` | One `use` per item. The PostToolUse hook formats files you edit; `cargo +nightly fmt --all` for the rest |
| A `self_named.rs` next to a `self_named/` directory | `self_named/mod.rs` |
| A new dependency line | Ask the user first. Declare it in the root `Cargo.toml` with `default-features = false` and use `dep.workspace = true` |
| Unused or pass-through arguments, superseded code left next to its replacement, commented-out code | Delete them |
| New logic with no test | A unit test with a one-line doc comment saying what it guards against |

## Too far: remove these too

- A trait with one implementation, or a generic with one instantiation that isn't a const-generic switch.
- A newtype that exists only to give another type a `Display` for one caller. That formatting belongs at the caller.
- Doc comments longer than one line on private items. `missing_docs_in_private_items` is satisfied by one line.
- Macros or "magic" that make the user-facing API shorter but hide what happens (root `AGENTS.md`, Design principles).
