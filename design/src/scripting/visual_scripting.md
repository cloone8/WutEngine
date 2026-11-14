# Visual scripting

A graph editor for non-programmers. This chapter covers what its graphs compile to and how errors find their way back
to nodes.

## Graphs compile to TypeScript

**A visual script compiles to TypeScript,** so there is one runtime, as a shader graph compiles to shader text
([Shaders and materials](../rendering/shaders.md)).
Why TypeScript rather than plain JavaScript:
- one pipeline: generated code goes through the same `tsc` check, emit, source maps, hot reload and error reporting as
  hand-written scripts ([Scripting](README.md));
- graphs and code mix: a graph can call a programmer's TypeScript functions with their real types, and code can call a
  graph's functions;
- the generated code is readable, so a graph can be turned into code when it outgrows the graph editor;
- plain JavaScript would save a compile step, but the graph editor would then need its own type checking for
  everything that isn't an engine type.

## Errors

- **Most type errors are prevented while connecting nodes:** pins are typed from the reflection schema.
- **A `tsc` error in generated code** then means a generator bug or a changed signature (a script function the graph
  calls was edited). Errors map back to nodes through a line-to-node table kept by the generator; runtime errors the
  same way after the source map step.

## The node editor

**The node editor widget is built in-house on egui,** first for the shader graph and generalized for visual
scripting ([Renderer](../rendering/README.md)).
