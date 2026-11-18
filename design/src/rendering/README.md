# Renderer

The engine's renderer, used by every game on every platform and by the editor, which draws its scene view with it.
This chapter covers the graphics API and GPU features, the renderer path, the rules that keep later techniques
possible, how shaders are authored, and the order features are built in. The structure is in [Renderer
architecture](architecture.md), shaders and materials in [Shaders and materials](shaders.md).

## Graphics API and GPU features

- **wgpu, WebGPU only** ([Principles and constraints](../goals/principles.md)). The WebGPU core is the baseline: it
  has compute shaders, storage buffers, single indirect draws, up to 8 colour attachments and MSAA 1 or 4, but no
  bindless, mesh shaders, ray tracing, multi-draw indirect or push constants.
- **Use any feature and limit wgpu exposes, always checked at runtime.** Shipped games request everything the GPU
  supports. Why: each platform gets its maximum while the engine stays compatible with the WebGPU baseline. Left out:
  - **experimental features:** wgpu requires an `unsafe` opt-in for them and documents them as possibly buggy;
  - **mappable primary buffers unless CPU and GPU share memory:** they only help where memory is shared and hurt
    badly where it isn't. So they are requested only on integrated GPUs and software (CPU) renderers, never on
    discrete or virtual GPUs or an unknown device type; where requested, code checks the feature before relying on
    it, as with any other;
  - **passthrough shaders, for now:** they bypass naga's validation, which the shader pipeline relies on
    ([Shaders and materials](shaders.md); a later use is in [Later](../later.md)).
- **Accelerators** (same output, faster: timestamp queries, pipeline cache, multi-draw indirect, subgroups) are used
  freely; **enhancements** (different output) need a baseline fallback.
- **A baseline switch:** a development tool (editor or development build) that requests only the WebGPU minimum, to
  see what the web and minimum hardware get. Why: it exercises the fallbacks enhancements must have.
- **Features that change shader code become `weshc` keywords;** the runtime picks the variant
  ([Shaders and materials](shaders.md)).

## Renderer path: incremental

- **Forward first, then clustered forward, then perhaps GPU-driven.** A deferred path is an optional later learning
  project. Cutting-edge techniques don't have to exist early, but no decision may block them later.
- **Why clustered forward over deferred:** deferred would be a second path next to forward, which stays anyway for
  transparency, shading models with a custom `light()` and MSAA; WebGPU core also caps a G-buffer at 32 bytes per
  sample. Clustered forward extends the forward path with two compute passes (cluster grid, light culling into
  per-cluster lists), and the fragment shader loops over its cluster's lights.

## Architecture rules that keep later techniques possible

1. **Shaders are surface functions, not full programs.** A shader describes its surface (albedo, normal, roughness,
   metallic, emission, alpha, optional vertex offset) against a shading model; the engine wraps it per pass (forward,
   depth and shadow, G-buffer, later ray tracing). The pass is a `weshc` variant axis next to keywords. Shader
   programs (full custom programs) stay as an escape hatch, working only in the passes they implement. A material is
   settings for either ([Shaders and materials](shaders.md)).
2. **Per-object and per-view data are kept apart as far as performance allows,** so as little as possible is uploaded
   (update on change): an object's model matrix (plus the previous one, for motion vectors) stored once, each view's
   camera matrices stored once, combined in the shader. The buffer layout is implementation. Why: one object in
   several views, motion vectors, and later a GPU-side scene.
3. **Views, not cameras.** Passes render a view: a viewpoint with a projection and a target. A player camera produces
   a view, but so do shadow cascades, reflection probe faces and render-to-texture captures.
4. **Shared mesh buffers.** Meshes live in a few large GPU buffers addressed by offset. Why: indirect and GPU-driven
   draws and later meshlets.
5. **A render graph** ([Renderer architecture](architecture.md)). Forward or deferred becomes a choice of passes in
   the graph; clustered light data serves both.

## Shader authoring

**A shader graph for artists is a major goal; programmers also write shaders in a text language.** The shader (a
surface function) is the shared layer:

| Who | Writes | Works in |
|---|---|---|
| Artist | Shader graph, which generates a WGSL shader | Every pass its shading model supports |
| Programmer | Hand-written WGSL shader | Every pass its shading model supports |
| Programmer (escape hatch) | Shader program: full WGSL with its own entry points | Only the passes it declares |
| Programmer, for artists | Custom nodes: a WGSL function asset the editor finds by itself | Any graph |
| Artist | Subgraphs: a graph saved as a function asset, used as a node in other graphs | Any graph |
| Both | Inline code node: WGSL typed into the graph, for one-offs | Where nodes fall short |

Why: hand-written shaders don't have to implement every pass, and graphs and text share one library.

Consequences:
- **Shading models are public API:** stable, changed deliberately, like UUIDs.
- **Compiler errors on generated WGSL map back to graph nodes.**
- **A shader graph is a text asset that `weshc` compiles without the editor.**
- **Material previews use only built-in engine features,** so the editor renders them itself.

## Custom nodes

**A node asset is a small WGSL file that the editor finds by itself and offers as a node** (the scripting and editor
layer may use such convenience, [Principles and constraints](../goals/principles.md)).
Why not script-defined nodes that generate code: compiling a graph would then mean running TypeScript, so `weshc` and
the build CLI would need boa and `tsc`.

- **One node per asset.** The file may contain several functions: one is the node, the others its private helpers.
  With a single function it is the node without setup; with several, the asset's settings choose. Helpers shared by
  several nodes go in a plain WGSL module the node assets import.
- **Pins come from the signature:** parameters are input pins, the return value the output pin; a returned struct
  gives one output pin per field. Texture and sampler parameters work, so sampling nodes do too.
- **The asset's settings file** sets the category, display name and pin defaults; the tooltip comes from the doc
  comment. No new WGSL syntax.
- **A graph references a node by the asset's UUID,** so renaming the file doesn't break graphs and the CLI compiles
  graphs without the editor.
- **Text shaders import node assets and shared modules like any WGSL module,** so artists and programmers share one
  library.
- **A subgraph is the graph-authored counterpart:** one node per asset, in the node menu the same way, compiled to a
  WGSL function.
- **Limit:** a plain function can't change its code based on node settings or connected types. Type variants (`f32`,
  `vec2`, `vec3`) are separate assets for now. Kept possible: one node asset holding same-named variants, picked by
  the connected type.

## The node editor widget

**Built in-house on egui,** for the shader graph first and generalized for [visual
scripting](../scripting/visual_scripting.md). egui provides pan and zoom, Bézier shapes and areas; pins, wires,
selection, the search menu, comments, reroutes and error markers are ours.
Why: the graph data model stays ours (asset format, undo, compiler errors mapped to nodes, typed pins), no dependency
gates egui upgrades, and no maintained crate fits.

## Feature order

The order features are built in, which is also the learning plan. Rules:
- **Simple before complex,** unless a more complex item heavily blocks others; the foundation is an explicit step 0.
- **2D goes in as early as possible,** as small additions next to the 3D work (3D stays the priority).
- **The first game** (PSX-style, unlit, some post-processing; [Why the engine exists](../goals/why.md)) sets what
  comes first.
- **Many lights through clustered forward;** deferred optional and later. MSAA before TAA.
- **Compute shaders early,** with their API designed: they feed clustered lighting, GPU-driven rendering, particles
  and compute post effects.
- **PBR later:** simpler lighting first, so shadows and other basics come earlier.

The order:

0. **Foundation:** render graph with injection points, views (perspective and orthographic), retained GPU scene,
   shared mesh buffers, shading-model wrapping and pass variants in `weshc`, debug labels, a linear colour workflow
   (sRGB textures, an `rgba16float` target per view, tonemapper None, sRGB output encoding). Unlit on top of it.
   Compute passes in the render graph and compute shaders in `weshc` from the start.
1. **Unlit complete, and 2D basics:** alpha clip and blending, a sorted transparent pass; vertex colours; a sky
   (cubemap or gradient); sprites (a Sprite shading model on a shared quad) with sorting layers and order in layer;
   texture import settings for pixel art (nearest filtering, no mipmaps). Enough for a simple 2D game. Plus the
   compute API outside the frame (GPU work with async readback).
2. **First-game kit:** fullscreen post materials at injection points; a view rendering into a low-resolution texture,
   upscaled with nearest filtering; distance fog; custom interpolators with WGSL interpolation qualifiers
   (`@interpolate(linear)` for affine textures); vertex-stage access to the clip-space position (vertex snapping).
   With these, the PSX look is a user shading model plus post materials. Enough for the first game.
3. **Simple lighting:** a built-in "Simple Lit" shading model (Lambert diffuse, Blinn-Phong specular; per pixel, or
   per vertex for a retro look), a small fixed number of directional, point and spot lights, ambient from a constant
   or the sky colour, physical light units and manual exposure (EV100), normal maps (tangents generated at import when
   missing), 4× MSAA, debug views (albedo, normals, overdraw). Simple Lit stays once PBR exists.
4. **Shadows:** a single directional shadow map, then cascades, then an atlas for spot and point lights.
5. **Many lights:** clustered forward with compute light culling.
6. **PBR:** the PBR Neutral tonemapper with the new-3D-scene defaults; a "Lit" shading model with PBR
   metal/roughness (glTF-compatible); image-based lighting (HDR environment import).
7. **Post:** bloom (may move earlier), auto exposure, SSAO; TAA with motion vectors as an option.
8. **GPU-driven:** compute culling writing indirect arguments, instancing. Tilemaps as instanced or batched chunks
   (or earlier as CPU-built chunk meshes if a 2D project needs them).

Later, optional learning projects: a deferred path, screen-space reflections, global illumination, volumetrics,
decals, 2D lights. Particles belong with VFX ([Later](../later.md)).
