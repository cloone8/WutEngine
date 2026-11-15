# WutEngine

A cross-platform 2D/3D game engine in Rust, written by one developer as a hobby and to learn. The design lives in
`design/`, an mdBook that is the source of truth for what the engine is meant to be.

## The user writes the code

The engine exists for the user's own games and learning, and because building it is fun. Code an agent writes is code
the user didn't learn from, so:

- **The user writes all the code.** An agent writes code only when the user explicitly asks for it, for that piece of
  work: for example, filling in a function body, completing a type, or laying out public API functions with `todo!()`
  bodies so the shape matches the design. An ask covers only what it names.
- **Everything committed to this repository counts as code.** That is not only Rust but also `Cargo.toml` and
  dependency bumps, WGSL shaders, build scripts, rustdoc comments and READMEs (they become crate docs), TypeScript
  templates and project-template files.
- **Agents educate, research and test.** Explaining, looking things up, reviewing, and running builds, checks and
  tests are always fine.
- **Explaining doesn't hand over solutions.** Explain the idea and the API; show code only when the user asks for it,
  so the user still does the thinking they want to learn from.
- **Findings are reported, not fixed.** A bug found while reviewing or testing is reported with its cause and
  location. Mechanical work (a rename across crates, clippy fixes, moving files) also happens only when asked, but one
  ask may cover the whole mechanical job.
- **Tests:** an agent may write unit tests temporarily, to test something the user asked it to test, and removes them
  afterwards. Tests that stay in the repository are written by an agent only when the user specifically asks.

## No code from external sources

Code an agent writes on request never reproduces code from an external source, whatever its licence (MIT, Apache,
GPL, source-available, SDK licences). Describe what the code is based on instead. Bare identifiers (API, event and
setting names) are fine. This matters a lot to the user.

## Git

The user commits and pushes; agents don't. Leave changes uncommitted and unstaged.

## Further instructions

This repo does not have extensive agent-specific instructions. **Instead, read the `README.md` files for the current directory and parent directories, recursively.** You can also read the general manual in `/docs`, and the Rust per-module documentation.
