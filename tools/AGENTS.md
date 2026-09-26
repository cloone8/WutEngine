# CLI tools

The tools make it possible to build a game without the editor (root `AGENTS.md`, Design principles), so they are
user-facing: clear `--help` text and error messages matter.

- `weimport`: source files (images, obj/mtl) to assets, in parallel.
- `weassetconv`: converts assets between text and binary.
- `weshc`: compiles a shader source into one `PrecompiledShader` asset per keyword combination
  (`-k KEY`, `-k KEY=3`, `-k KEY=0..4`), using `wutengine_shadercompiler`.
- `wutengine_cli_tools`: what they share.

A tool with a GUI is a WutEngine game, like the editor (root `AGENTS.md`, Design principles).

## Conventions

Follow the existing tools when adding one or an option:

- `clap` derive with `styles = wutengine_cli_tools::clap::STYLING`, and `OutputFormatArg` for `--text`/`--binary`.
- Input from a file or `--stdin`, output to a directory or `--stdout`, as mutually exclusive required groups.
- Log with `simplelog` to stderr, level set by `-v`/`--verbosity`; stdout carries only the output.
- `main` returns `ExitCode`: log the error and return `ExitCode::FAILURE` instead of panicking.
- Throttle parallel work with `wutengine_util::JobQueue`.

## Known gaps

`weassetconv` detects the asset type by trying every registered type's deserializer in `HashMap` order, so an
ambiguous file can come out as a different type between runs. Its `-o` help says directory, but it's used as a file
path.
