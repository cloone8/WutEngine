# Editor

`wutengine_editor` is the editor application; its binary is named `wutengine`. It runs on Linux, macOS and Windows
only, so desktop-only code (files, threads, `rfd` dialogs) is fine here, unlike in `runtime/`.

The editor is a regular WutEngine game, not an eframe app: `main.rs` calls `wutengine::runtime::start` and builds its
UI from entities and components, drawn with egui through `wutengine_egui` (`editorwindow_renderpass.rs`). Anything the
editor needs from the engine should go through the same public API a game would use; if that API is missing
something, extend the engine rather than working around it here.

- Windows implement `EditorWindow` (`src/window/`); the main window holds panel containers.
- Panels implement `EditorPanel` (`src/panel/`): tree, log, library, and a placeholder test panel.
- Menus use `we_menu`; colours `we_style`; fonts `we_fonts` (see its README for the font licences); per-user
  preferences `we_prefs`.
- Projects (`src/project/`): a `<name>.we-project` file, an `assets.json` index of asset UUIDs to files, and an
  `assets/` folder. Assets are loaded through the editor's own asset server (`src/assets/server.rs`).
- Creatable assets implement `CreateAsset` (`src/assets/create.rs`).

Launching the editor opens a window: ask the user first. `cargo run -p wutengine_editor -- [project.we-project]`;
`--help` lists the options.
