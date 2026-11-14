# Shaders and materials

How shaders are structured and written, how they are compiled, varied and stored, what a material holds, how textures
are compressed per platform, and what happens when a shader fails. Who writes which kind of shader is in
[Renderer](README.md).

## Terms

| Level | Name | Written by | What it is |
|---|---|---|---|
| What gets shaded | **domain** | Engine | Surface, fullscreen; later sky, particles, sprite and UI, decals |
| Interface and per-pass wrapping | **shading model** | Engine and users | Inputs, outputs with defaults, supported passes, per-pass entry points |
| Logic against a shading model | **shader** | Artist (graph) or programmer (text) | `fragment()` and so on against a shading model; declares parameters; the engine generates the programs per pass |
| Full program (escape hatch) | **shader program** | Programmer | Own entry points; works only in the passes it declares |
| Settings | **material** | Anyone, in the inspector | Parameter values, textures and render state where the shader allows it, for a shader or a shader program |

## Domains, shading models and where a shader runs

- **Domains are engine-defined,** few in number: they need renderer machinery, so a new domain means writing render
  graph passes in Rust.
- **Shading models are definable by the engine and by users,** per domain, with graph and text shaders on the same
  shading model. Examples: surface Unlit, Lit, Toon; fullscreen HDR post and LDR post. A shading model declares
  per-stage inputs, outputs with defaults and the passes it supports, and contains the per-pass entry points that call
  the shader's functions. Why: programmers can expose new looks to artists without engine changes.
- **A shader targets a shading model** in text (a `#model "Unlit"` directive and `fn fragment(...)`) or in the graph
  (the output node's pins are the shading model's output fields; parameters are the detected bindings).
- **Opaque or transparent is render state on the same shading model,** set in the material where the shader allows
  it. Render graph passes select materials by supported pass and render state; "transparent after opaque" is pass
  order in the graph.
- **Fullscreen shading models declare which injection points they can run at;** a camera component or a custom pass
  places the materials ([Renderer architecture](architecture.md)). One built-in render graph with fixed injection
  points, which are also where programmers plug in their own passes.
- **Lighting customization, both ways:** built-in lighting models for most shaders, and user shading models may
  contain a custom `light()` (toon shading, stylized rims, ramps). A shading model with custom lighting declares no
  G-buffer pass by default, so it renders forward. Kept possible: a later deferred path generating its lighting pass
  from all shading models' `light()` functions, as long as their inputs fit the G-buffer.

## Shader outputs

**A shader stage writes through a pointer to a pre-filled output struct:**
`fn fragment(in: LitFragmentInput, out: ptr<function, LitFragmentOutput>)`. The generated entry point creates the
shading model's output struct with its defaults, passes the pointer, and reads the struct back afterwards.
Why: defaults are automatic (WGSL struct members can't have initializers, and zero is a wrong default: black,
transparent, a zero normal); the types keep stages apart (`vertex()` can't set fragment outputs); and data flow stays
explicit. If `out.field` through a pointer isn't usable on the web, `(*out).field` is accepted.

## The text language

**WGSL plus `weshc`'s own directives,** implemented in `weshc`, with WESL-compatible syntax for imports and
conditional compilation (`import package::module;`, `@if(...)`), so files stay familiar and can benefit from WESL
tooling. WutEngine-specific directives (choosing a shading model) are our own additions; shading-model interfaces are
structs from the engine's import.
Why: no outside crate for an engine core part, no large C++ compiler, and no language users know from nowhere else.

## Graph compilation

**A shader graph generates the same WGSL `fragment()` a programmer would write,** which then goes through the same
`weshc` path. The output is readable (diffable, convertible to a text shader); an inline code node is pasted text;
compiler errors map back to nodes through a line-to-node table.
Why: one compiler path, as visual scripting compiles to TypeScript; building compiler IR directly would be a second
front-end tied to changing internals, with unreadable output.

## Variants

Variants are an optimization, not a correctness requirement.

- **Keywords only where bindings or interfaces change;** uniform branches or WGSL `override` constants (fixed at
  pipeline creation) for other toggles.
- **Builds pre-generate the variants their materials use.** A variant missing at runtime (a shader switched at
  runtime, a keyword toggled, a material created dynamically) is generated on first use from the shader sources
  shipped with the game, by the same step as the editor's on-demand compilation: a hitch, but correct rendering.
  Development builds warn (add the material, or prepare it at load); release builds log it. So no split between
  "material" and "runtime" keywords is needed. Platforms without runtime shader compilation render an error shader
  instead. Why: an error shader as the general fallback gives "works in the editor, broken in the build" bugs.
- **Shaders follow the asset inclusion rule:** included if a material in the build references them, or marked "always
  include" ([Building and shipping](../editor/shipping.md)).
- **Pipelines a scene needs are created while the scene loads,** off the main thread on native; runtime-loaded content
  creates its pipelines on its async load path. A pipeline cache where supported.
- **What can't be promised:** GPU pipelines are always created on the player's machine; the driver compiles for the
  player's GPU. First-use stutter is fought with the steps above, not eliminated.

## Stored form of compiled shaders

- **Builds store each pre-generated variant as a binary naga module;** the editor works from the shader sources.
  Compiled shaders are regenerable data (import cache in the project, made by the shipping engine in a build), so
  naga's internal format changing between versions is no risk. Runtime-generated variants go from WGSL text through
  naga in the player.
- **Web builds store preprocessed, validated WGSL text instead.** Why: browsers accept only WGSL strings, and
  converting naga modules to WGSL in the player would add download size.

## A shader that fails to compile

**naga validates every shader before wgpu sees it,** since a wgpu validation error counts as a failed assert
([Error handling](../core/errors.md)).
- **A build with a broken shader fails,** with the error mapped to the graph node or source line.
- **In the editor and play mode, a shader that never compiled draws the error material** and logs the error
  ([Assets and project files](../world/assets.md)).
- **On hot reload, the last version that compiled stays,** with the error shown in the console and on the shader. Why:
  no magenta flash while typing (an inline code node edited during play mode).
- **In a shipped game every variant was validated at build,** so a failure is an engine bug: logged as one, drawn with
  the error material.

## Material parameters

1. **What counts as a parameter:** everything the shader declares in the material bind group. Shading models and
   engine imports own the other groups (view, object). Numbers and vectors go in one uniform struct; textures and
   samplers are their own bindings. Why: one uniform buffer per value would hit binding limits.
2. **Metadata as custom attributes** on the struct fields and bindings (`@color`, `@range(0, 1)`, `@default(0.5)`,
   `@category("Surface")`, `@default(white)` on a texture, `@sampler_for(albedo)` on a sampler), which `weshc` reads and
   strips before naga. Doc comments stay documentation and become the inspector tooltip. `weshc` reports unknown or
   misspelled attributes as errors. Why: our files need `weshc`-aware tooling anyway, attributes read better than
   annotations in comments, and defaults and ranges belong with the code that relies on them. Risk: a future WGSL or
   WESL attribute with the same name means renaming ours.
3. **Widgets come from type plus annotation:** numbers, sliders (`@range`), vectors, colour pickers (`@color`,
   `@hdr`), dropdowns (`@enum(...)` on integers), toggles (`@toggle` on `u32`, since WGSL uniforms can't hold `bool`),
   `@hidden`, inspector sections (`@category`; not `@group`, which WGSL uses for bind groups), and the display name
   from the field name unless `@name(...)`.
4. **Colour space:** `@color` values are edited as sRGB and converted to linear when written; textures carry sRGB or
   linear in their import settings.
5. **Samplers paired with `@sampler_for` follow the texture's import settings** (filtering, wrap); an unpaired sampler
   is a material parameter with filter and wrap in the inspector.
6. **Keywords and render state in the same inspector:** keywords as toggles or dropdowns (choosing the variant),
   `override` constants as values that pick a pipeline, and the render states the shader allows (blend, cull, depth,
   render order).
7. **Set from code by name** (`material.set("tint", …)`), checked against the reflection, as compute binding is.
   Shader graphs generate the same struct and attributes from their parameter nodes.
8. **Material files store values by name.** Switching a material to another shader keeps values whose names match;
   values the current shader doesn't have are kept, not dropped, so switching back loses nothing.

## Texture compression: per-platform builds

**Asset formats are selected at build time per platform; a platform's package never ships what it can't use** (this
holds for every asset, not only textures). The exception is the web, where one build meets desktop GPUs (BC) and
mobile GPUs (ETC2, ASTC): it includes several variants, and the browser downloads only the one its GPU needs
([Building and shipping](../editor/shipping.md)).
Why: each platform gets its native format at full quality, without a universal transcoded format.
