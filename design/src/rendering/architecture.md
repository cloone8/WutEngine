# Renderer architecture

The renderer's structure: the render graph and its extension points, the retained GPU scene, compute, the colour
pipeline and light units, texture binding, the mesh format, visibility and draw order. The rules behind it are in
[Renderer](README.md); depth (reversed Z, infinite far plane, `depth32float`) is in [Conventions and
names](../core/conventions.md).

## The render graph

The frame is a render graph: per frame, setup (passes declare what they read, write and create), compile (cull unused
passes, compute resource lifetimes, validate), execute (record; transient textures come from a pool and are reused
when lifetimes don't overlap). wgpu handles barriers itself and has one queue, so the graph's jobs are ordering,
culling, resource lifetimes and pooling, and being the extension point.

- **The graph is rebuilt every frame;** cached only if profiling says so. Why: cheap and fully dynamic.
- **One graph per frame contains all views.** Why: shared work such as shadow maps and editor previews.
- **Passes run in the order setup adds them.** The built-in setup walks the stages and calls registered custom
  passes at each injection point. Why not order purely by declared dependencies: many passes modify one resource in
  place (scene colour: opaque, sky, transparent, fog, bloom, tonemap, UI), so intent isn't in the data; several valid
  orders would be unstable; and user passes would silently reorder when built-in internals change.
- **Between views, order follows dependencies:** a view that renders into a texture another view reads goes first, so
  a camera sampling another camera's render texture always sees this frame's image. **A cycle** (two facing mirrors,
  a monitor visible to its own camera) reads the previous frame at the edge that closes it.
- **A pass is a setup step that declares what it reads, writes and creates, and returns the execute closure** that
  records commands. Writing a resource returns a new version of its handle, so the order of writers is explicit;
  using an outdated version is caught when the graph is built. Why: the same "declare, then the engine schedules"
  pattern as Rust systems.
- **One command encoder per pass,** small passes not grouped. Independent passes record in parallel on native and
  sequentially on the web; each encoder's `finish()` runs on the worker that recorded it. **Command buffers are
  always submitted in graph order,** never in the order threads finish. Pass closures must be `Send` and not touch
  main-thread-only state. Kept possible: render bundles to record one heavy pass in parallel chunks.
- **Parallel recording assumes few bind-group changes per draw,** which the per-object buffers and shared mesh
  buffers provide. Why: in wgpu, a bind-group change per draw serializes the threads.

## Injection points and named resources

- **Injection points say when:** named points between built-in stages. Per view, such as `AfterDepth`, `AfterOpaque`,
  `BeforeTransparent`, `BeforeTonemap`, `AfterTonemap`, `BeforePresent`; per frame, one before all views (compute, GPU
  particles) and one after all views (compositing).
- **Named resources say with what:** per point, the engine guarantees which resources exist, their contents and
  format (at `AfterOpaque`: `SceneColor`, HDR, with opaque and sky; `SceneDepth`). Custom passes read and modify
  them, create their own, and publish new named resources for later passes.
- **Points and names are public contract,** like shading models; the built-in passes behind them may change. A
  deferred path keeps `SceneColor` and `SceneDepth` at `AfterOpaque` and adds `GBuffer`.
- **Resources on request:** a pass declares a need (normals, motion vectors) and the producing pass turns on; culling
  drops what nobody requests.
- **Draw by pass name:** shading models declare pass names, and any graph pass can draw "all visible objects whose
  material supports pass X" (a user "Toon" shading model with an outline pass, plus a custom pass at `AfterOpaque`).
- **Registration is explicit,** global (a plain function) or per camera (a component). Editor gizmos, grid and
  selection outline are per-camera custom passes, which dogfoods the API.
- **A missing point is added to the engine;** replacing a built-in stage comes later.
- Why named points rather than "after pass X": rearranging built-in passes must not silently break registered ones.

## The retained GPU scene

- **The renderer owns a retained scene** (per object: world matrix, mesh, material, bounds; plus lights and views) in
  GPU buffers, with CPU mirrors where needed. **One sync point per frame, before rendering, applies only what
  changed** (new, removed, moved, changed objects; the transform pass's dirty marking says what moved). **Render passes
  read only this scene, never the ECS.**
  Why: no coupling to the ECS, no borrow conflicts with parallel systems, and unchanged data isn't re-uploaded. Cost
  accepted: change tracking and render data held twice.

## Render graph tooling

In this order: **debug labels from the start** (every pass, texture and buffer carries its graph name, so RenderDoc,
PIX and Xcode show it); then, as the renderer grows, a graph view in the editor (passes, reads and writes, culled
passes), an intermediate texture viewer (channel and range controls for depth and HDR), GPU time per pass through
timestamp queries where supported, and an editor frame debugger stepping through the frame pass by pass. None of it
affects the architecture: it reads what the graph already knows ([Profiling and debugging
tools](../editor/debug_tools.md)).

## Compute

Compute is in the render graph from feature step 0 and available outside the frame from step 1
([Renderer](README.md)).

1. **A compute shader is an asset:** a WGSL file with one or more `@compute` entry points (kernels), compiled by
   `weshc` with the same imports, keywords and variants as other shaders. The workgroup size lives only in the shader
   (`@workgroup_size`, optionally an `override` constant).
2. **Binding by name, from `weshc` reflection,** so it also works on the web, where browsers give no WGSL reflection.
   Arguments are a value built per dispatch, never state on the kernel object. Names and types are checked against
   the reflection when the dispatch is recorded, with a clear error. Why per dispatch: two dispatches in a frame must
   not share the last values set.
3. **Buffer layouts from a Rust derive, checked against the shader.** The derive computes WGSL's storage layout
   (alignment and padding) from the Rust type; at bind time it is compared with the struct `weshc` reflected, so a
   mismatch is an error instead of garbage. The derive is our own for now.
4. **Dispatch by size:** a dispatch takes a thread count (or "the size of texture X") and the engine divides by the
   shader's workgroup size; explicit group counts and indirect dispatch stay available.
5. **One mechanism, two entry points:** inside the frame, a compute pass in the render graph (the same setup and
   execute shape as render passes, at injection points); outside rendering, **GPU jobs** that Rust code queues at any
   time, recorded at the per-frame injection point before all views of the next frame, in submission order. Why: a
   deterministic order, identical on the web.
6. **Readback is async only,** also on native: a readback returns a handle whose result arrives at frame start of a
   later frame, as loaded assets do ([Time and the frame](../core/frame.md)). The engine owns device polling and a
   staging-buffer pool. Latency is at least one frame, documented. Why: WebGPU has no synchronous mapping.
7. **Access:** Rust gets everything first. Scripts get GPU jobs with a promise-based readback later. Artists reach
   compute through graphs that generate it (VFX), not through a node that dispatches a kernel.
8. **An engine compute library** of WGSL modules and ready-made passes for the recurring building blocks (prefix sum,
   stream compaction, writing indirect arguments; radix sort later), shared with the engine's own culling and
   clustered lighting.

Bindings are checked by name at runtime first; generated typed Rust bindings may come later if Rust users want them.
Why: names are needed anyway by the editor, scripts and graphs, and generation would add a code-generation step to
every Rust project.

## Colour pipeline

- **Linear, Rec.709 primaries.** Colour textures are stored in an `-srgb` format (set by their import settings) and
  decoded when read; data textures (normals, roughness) are linear; glTF vertex colours are linear. Colours typed in
  the editor are shown and picked as sRGB and stored linear.
- **One pipeline, always HDR:** every view's colour target is `rgba16float`, from feature step 0. Why: one path to
  maintain, not a second LDR path with tonemapping folded into materials; at the first game's low resolution the cost
  is small. Kept possible: a low-end option without the HDR target.
- **Tonemapping belongs to the target, not the camera:** the cameras on one target draw in their order into one
  shared linear HDR buffer, and the target's tonemapper runs once, after its last camera. The target's clear colour
  fills the buffer once; each camera's background ([Input and camera](../systems/input_camera.md)) is then a draw
  limited to its viewport, since a pass's clear always covers the whole attachment. **A render texture stays
  linear** by default (tonemapper None, no encoding), so a material sampling it gets linear values and nothing is
  tonemapped twice.
- **The order per target, fixed now so later features slot in:**
  1. the cameras, in linear HDR, with their post materials at `BeforeTonemap` (bloom when built);
  2. grading, when built: before tonemapping, as one 3D LUT with a shaped (log) input; grading before tonemapping is
     what lets HDR output reuse it;
  3. tonemapping;
  4. `AfterTonemap` post materials (the "LDR post" shading model, display-referred values: the first game's colour
     reduction);
  5. output encoding (sRGB; later the HDR encodings), with dithering against banding when quantizing to 8 bits,
     **except with tonemapper None**: 8-bit sRGB colours survive the round trip through `rgba16float` exactly, and
     noise would spoil exact pixel art;
  6. UI drawn after the last camera, **blended in the encoded (sRGB) values,** as image editors and browsers blend. A
     UI screen ordered before a camera draws into the linear buffer and is part of the scene.
- **Surfaces never use an sRGB format:** the output pass encodes in the shader, so dithering controls the
  quantization and UI blends in encoded values, and the web's canvas offers no sRGB format anyway. The format, bit
  depth included, follows the output mode and isn't fixed to 8 bits: SDR now picks one of the surface's non-sRGB
  formats; HDR output later picks its own (a float format with its colour space). If a surface offers only sRGB
  formats, a non-sRGB view of it is used where wgpu supports surface view formats; otherwise the shader writes linear
  values for the hardware to encode, still dithered, and a warning is logged once that UI blends differently there.
- **A target that only UI draws to** (in-world UI: a terminal screen, a sign) is an sRGB-encoded 8-bit texture, its UI
  blended as on a window; materials sample it as an ordinary sRGB colour texture, so it is linear in the world, lit or
  unlit, and tonemapped with the scene. UI drawn straight into the 3D scene (damage numbers, name tags) is transparent
  geometry, blended in linear ([In-game UI and text](../systems/ui_text.md)).
- **Tonemappers:** **None** (exact, clamped) is the engine default, right for unlit and 2D. **Khronos PBR Neutral**
  comes with the new-3D-scene defaults: it reproduces the base colours of lit dielectrics up to about 0.76, but
  subtracts up to 0.04, so unlit colours would come out slightly darker. Kept possible: AgX and ACES.
- **Kept possible: HDR display output,** after PBR. The order above keeps it open (tonemapping and encoding per
  target, grading before them, UI at a paper-white level in the output's encoding).

## Physical light units

- **Always physical, no switch:** directional lights in lux, **point and spot lights in lumens** (glTF's candela
  converted at import), emissive and sky in nits, exposure as EV100 on the camera: manual from feature step 3, auto
  exposure later. Why now: changing units later changes the brightness of every existing scene.
- **Exposure is a shader contract, not a post step:** the engine's wrapper applies it to the output of shading models
  that declare it; lit shading models do (emissive included), unlit ones don't. An unlit material with tonemapper None
  shows texture colours exactly.
- **New 3D scenes look right out of the box:** the default sun, sky, camera exposure and the Neutral tonemapper are
  chosen together (a sunny-day EV100 and a matching sun), and the editor shows exposure next to the light values.

## Texture binding

- **Baseline (WebGPU core, every platform):** a material's textures and samplers are its bind group; per-object data
  is indexed per instance; draws are sorted to minimise changes.
- **Kept possible: binding arrays and bindless as an accelerator** on native GPUs that have them, and for GPU-driven
  rendering; never on the web until a proposal ships. **The shader contract doesn't change:** shaders keep declaring
  their material parameters, and `weshc` generates a binding-array variant, an engine-chosen axis not shown in the
  inspector, like the pass axis.
- **Texture arrays are an asset type** (layers of one size and format) for terrain layers and data. **Sprite frames
  are atlas regions,** not array layers.

## Mesh and vertex format

- **Positions in their own stream** in the shared buffers. Why: depth, shadow and picking passes read only them.
- **Formats are chosen per attribute at import and decoded by the engine's vertex wrapper,** so shaders always see
  decoded values (and a PSX shading model can still snap positions). Known traps the formats must avoid: UVs as half
  floats where textures exceed 2048 texels; vertex colours as 8-bit linear (dark gradients band).
- **UV1 always has a place:** glTF's second texture coordinate set is imported, and a light-map unwrap at import comes
  with baked lighting.
- **LODs inside the mesh asset:** index ranges over shared vertices, **vertices ordered coarsest LOD first,** so each
  LOD's vertices are a prefix and coarse LODs can stream alone; generated at import by simplification, selected per
  view by screen size with hysteresis.
- **Skinning and morph targets in a compute pass** before the passes that draw them: morph deltas from a storage
  buffer, then skinning (four influences), written into per-instance output in the shared buffers. **The previous
  frame's output is kept when a pass requests motion vectors,** as the previous model matrix is.

## Visibility

- **One culling step per view** (camera, shadow cascade, cube face, probe, capture), producing a visible list per view
  and pass name, with per-view options: shadow cascades cull without a near plane (casters behind the camera still
  cast); occlusion culling, when it exists, only for main views.
- **Brute force over the retained scene's bounds, split into jobs;** a spatial structure only after measuring. The
  renderer owns its culling structure; none is shared with physics or gameplay.
- **Kept possible: render layers per view** (a first-person weapon camera, a minimap), named apart from collision
  layers. Editor gizmos aren't render layers: they are per-camera custom passes.

## Draw order

- **One integer sort key per pass's draw list,** sorted once per view (opaque and transparent are separate passes):
  **render order** at the top, then for opaque passes pipeline and material before coarse depth buckets, front to
  back; for transparent passes depth back to front.
- **Render order is one concept:** a material's render state (an integer, for effects that must draw in a fixed
  order), and for sprites their sorting layer and order in layer, in the same field.
