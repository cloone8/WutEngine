# Packages

How reusable content and code (assets, TypeScript, shaders, Rust) is shared between projects. There is one kind of
package; registries, stores and sandboxing are kept possible, not built.

## One kind of package: a folder in the project

- **A package is a folder in the project's `packages/` folder** with a small **package file**: its name, version,
  origin, minimum engine version and dependencies, written by the package's author. It can hold assets with their
  settings files, TypeScript (scripts and editor extensions), shaders, and a Rust crate.
- **One concept,** not plugins next to packages next to add-ons: what a package contains decides what it needs. A Rust
  crate makes the project need the project-built editor and player, as project Rust code does; the "add package"
  dialog says so.
- **A package name** is lowercase letters, digits and `_`. Why: the same name then works as the folder, the import,
  the WESL package, the crate and a configuration table, on case-insensitive file systems too.
- **Copied in and committed with the project.** The editor and a CLI command add one from a folder or a zip; the
  package folder stays identical to what was added. **No registry and no downloads at build time.** Why: it works
  offline, in git and in the web build with nothing to restore, and the user can read and change it.
- **Adding a Rust package writes two visible lines:** the Cargo path dependency and the registration call in the game's
  entry point, both in the diff; removing the package deletes them. Why: registration stays explicit; the editor only
  writes it ([Principles](../goals/principles.md)).
- **Updating** replaces the folder from the newly added copy, after warning that files in `packages/<name>/` will be
  replaced and git shows the changes. Adding a package whose name is already present is an update only if its origin
  matches; otherwise the editor asks.
- **Updating and removing are file operations:** not undoable, removed files go to the OS's trash, and open documents
  from the package reload as after a git checkout ([Editor](../editor/README.md)). After a removal, references into the
  package fail as missing assets do, and imports of it become type errors.
- **Packages add no configuration layer.** A Rust package's typed section gets its defaults from its type's `Default`;
  a script package reads `[game.<name>]` values with a fallback in code ([Configuration](../core/configuration.md)).
- Kept possible: adding from a git URL; a declared, fetched form; a package configuration layer.

## Asset identity: a package's UUIDs are its contract

- **Package assets keep their UUIDs** (their settings files travel with them), so a scene in a package, or a project's
  reference into one, survives copying the package to another project and updating it.
- **A package's asset UUIDs are part of its public contract:** the manual tells package authors never to regenerate
  them; an update that removes or changes one breaks references like a removed API.
- **One copy of a package per project.**
- **A duplicate UUID anywhere in the project** (files copied by hand) is found when the editor or the build tools read
  the settings files: the editor names both files and the user picks which gets a new UUID; the build fails until it is
  resolved. Never resolved silently.
- **Builds treat package assets and scripts like the project's own** (what's referenced, plus "always include").

## Code joins like project code

- **TypeScript:** a package's scripts are imported by the package's name: `"<name>"` is `packages/<name>/index.ts`,
  `"<name>/..."` a file inside the package. One rule, fixed in the engine, serves `tsc` (through the generated
  `tsconfig.json`) and boa's module loader. Why: what type-checks is what runs, and no per-project table is compiled
  into the player, so prebuilt players stay possible. The engine's own module names are reserved, so a package can't
  shadow them. Kept possible: npm packages (plain JavaScript libraries).
- **Shaders:** a package's shaders form the WESL package of the same name; `package::` inside it means the package's
  root ([Shaders and materials](../rendering/shaders.md)).
- **Rust:** a package's crate is a Cargo path dependency of the project, compiled and statically linked with it, and
  uses the project's engine dependency rather than naming its own engine version. Why: Cargo then never builds two
  engines whose types don't match. Ordinary crates from crates.io stay ordinary Cargo dependencies.
- **No dynamic plugins.** Why: Rust has no stable ABI, and the web can't load code.
- **Editor-only parts** are marked the same way as the project's editor-only code ([Editor](../editor/README.md)).

## Versions and dependencies: checked, not resolved

- **The package file lists dependencies** (name, minimum version, where to get it) and the minimum engine version, the
  package's only statement of an engine version. The editor and the build tools check them when a package is added and
  when the project opens or builds: a missing or too-old dependency, or a too-new engine requirement, is reported by
  name, with an offer to add the missing package from where its entry says.
- **Before 1.0, a different 0.x minor engine version is a warning.** Why: 0.x promises no compatibility, and Cargo
  treats a 0.x minor bump as breaking.
- **No resolver:** one version per package per project. Kept possible: version ranges, conflicts between packages that
  want different versions, a registry.

## Trust: adding a package runs no code

- **Nothing runs while a package is copied in** (no install scripts). Its editor extensions and edit-mode scripts run
  when the editor loads them; its Rust code, with its `build.rs` and proc macros, at the next compile of the editor or a
  build. From then on its code runs with the same rights as the project's own; the manual and the "add package" dialog
  say so.
- Kept possible: sandboxing package code.

## Engine features aren't packages

- **Built-in features are versioned and shipped with the engine,** not as first-party packages with their own release
  cycles. Switching unused features off for smaller builds is a build setting, designed when built.
- Kept possible: the engine shipping its templates (the ready player, the save script) as packages copied in on use.
