# Shader compiler

Compiles WutEngine shader sources (WGSL plus preprocessor directives) into `PrecompiledShader` assets
(`asset/wutengine_assets/src/assets/shader.rs`). Used by `weshc`, and by `wutengine_graphics` for variants that weren't
precompiled.

## Pipeline

Parsing a `ShaderInfo` (`FromStr`) reads the `#name` and `#keyword` directives and hashes the source.
`ShaderInfo::variant` fills in default keyword values, rejects undeclared or out-of-range ones, and hashes (source hash
plus sorted keyword values, xxh3-128) into a `Variant`. `compile` (`src/lib.rs`): preprocess → parse with naga's WGSL
frontend → validate with `naga::valid::Validator` → in parallel, `bindings::find_bindings` and
`vertex_inputs::find_vertex_inputs` → `engine::check_layout` → `PrecompiledShader`. `compile_multiple` compiles every
combination of keyword ranges on rayon, throttled by `JobQueue`, and streams the results through a channel.

## Source format

Lines whose first non-blank character is `#` are directives (grammar: `src/preprocessor/grammar.pest`):

```wgsl
#name "Unlit"
#keyword HAS_COLOR_MAP
#keyword QUALITY 0..3
#import "wutengine"
#if HAS_COLOR_MAP != 0 && !QUALITY
@location(1) uv: vec2f,
#else
#endif
@group(WUTENGINE_MATERIAL_GROUP) @binding(0) var<uniform> params: Params;
```

- `#name` is required and must be the first directive. `#import` resolves another shader by name, once per shader. A
  directive line holds nothing else: a trailing `//` comment fails to parse.
- `#keyword NAME` allows 0 and 1; `#keyword NAME a..b` / `a..=b` gives a range. The default is the range start.
  Keywords are declared only in the main file; an `#if` on an undeclared keyword is an error. Values are `u64`;
  expressions support `! == != < <= > >= && ||` and parentheses; numbers can be decimal, `0x` or `0b`.
- Declared keywords are also substituted by value in source lines, whole words only.
- `#import "wutengine"` is built in (`src/engine.rs`, `src/wutengine.wgsl`): the `WUTENGINE_*_GROUP` WGSL constants
  and the camera (group 0) and instance (group 2) bindings. Material parameters go in `WUTENGINE_MATERIAL_GROUP`
  (group 1). Groups 0 and 2 may only hold the engine's bindings.
- Bindings are the uniform, storage, texture and sampler globals that an entry point uses; unused ones are left out.
  Each needs a name and a `@group/@binding`. A struct buffer is flattened one level into members.
- Vertex inputs come from the single `@vertex` entry point, by name: `position`/`pos`, `normal`, `color`, and
  `uv`/`texcoord`/`tex_coord` with an optional channel number (`uv1`).

## Known gaps

- The variant hash covers the main source only, not imports: editing an import doesn't invalidate precompiled
  variants.
- Only the built-in import resolves in the runtime and in `weshc`; neither has a resolver for user shaders yet.
- The runtime uses a precompiled variant only after `wutengine_graphics::shader::register_precompiled`; nothing loads
  `PrecompiledShader` assets from disk yet, so every variant is compiled from source.
- `find_bindings` rejects runtime-sized arrays, storage textures and binding arrays.
