# Reflection

How the engine describes types and values at runtime, for the inspector, serialization, the editor link, scripting
and configuration.

## What reflection is

- **Reflection is serde for values plus a derived schema for types and metadata,** as data that can cross the editor
  link. serde turns a value into a self-describing tree (field names, values) and back, which covers editing, saving,
  sending over the editor link, diffs and undo. The schema adds what serde can't give: a type's shape without a value
  (all variants of an enum for a dropdown, the element type of an empty list) and metadata (ranges, tooltips, hidden
  fields, "this is a colour").
- **The schema comes from a derive on the Rust type,** not from hand-written descriptors or types defined as data.
- **Functions are exposed to scripts separately,** by explicit registration ([Script API](../scripting/api.md)).
- **What uses it:** the inspector; scene serialization (with overrides on scene instances, undo and diffs through
  field-level paths); the editor link; scripting and visual scripting bindings and the generated `.d.ts` files; origin
  shifting (finding position fields); configuration; project compatibility (renamed or removed fields).

## Script-defined data

- **A script attached to an entity declares its own properties** with `@export`, as in Godot. They appear in the
  inspector and are saved with the scene, described in the same schema format as Rust components, so the editor treats
  both the same. Why: scripts need data of their own for the Unity-like editor experience the engine aims at.
- **Rust systems can't query script data by type;** other scripts can. Kept possible: fully dynamic component types,
  so script components become queryable from Rust; describing script properties in the same schema format keeps that
  open.

## Registration

- **Explicit:** the game's entry point lists its component types; the derive only produces the schema. A scene that
  references an unknown type UUID gives a clear error. Why: automatic registration is hidden magic, which the Rust
  side rules out ([Principles](../goals/principles.md)).

## Schema evolution

- **Fields by name;** serde's `alias` for renames and `default` for new fields; per-type versions with migration
  functions only when needed.
- **Unknown fields are kept** instead of dropped on re-save. Why: serde ignores them by default, so a teammate on an
  older build would erase data. This only works for self-describing formats, hence text project files
  ([Assets and project files](assets.md)).
- During major version 0 there is no compatibility promise at all ([Principles](../goals/principles.md)).

## Typed field handles

The derive gives one constant per field (`GraphicsSettings::VSYNC`), used for per-field configuration locks, sets and
change checks ([Configuration](../core/configuration.md)). String paths are the accepted fallback.
