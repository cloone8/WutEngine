# Asset crates

- `wutengine_assets`: the serialized asset types (`src/assets/`) and the `SerializedAsset` / `FromSerializedAsset`
  traits (`src/lib.rs`).
- `wutengine_asset_importers`: turns source files into assets (`image.rs` for textures, `obj.rs` for meshes and
  materials). The `generic` feature adds type-erased registries (`default_importers`, `default_asset_types` in
  `src/generic.rs`) used by the CLI tools and the editor.
- `wutengine_asset_server`: loads assets by ID through an `AssetLoader`, caches them, and hands out `TaskHandle`s.

## Formats

An asset is serialized as binary (postcard, `.we-binasset`) or text (pretty JSON, `.we-txtasset`). The format used
to load an asset is decided by `PREFER_BINARY_SERIALIZATION` or the loader, not by the file extension. Assets reference
each other with `AssetRef<T>` (an asset UUID).

The CLI tools must be able to do everything the editor does with assets (root `AGENTS.md`, Design principles).

## Compatibility

Project files on disk store these types. A renamed or removed field, a changed enum representation or a changed
`ID` breaks existing assets. Keep a renamed field loadable with `#[serde(alias = "old_name")]`, give new fields a
`#[serde(default)]` where that makes sense, and never change an existing `ID`.

## Adding an asset type

1. A struct in `wutengine_assets/src/assets/` implementing `SerializedAsset`, with a freshly generated v4 UUID as
   `ID` (`uuid::uuid!("...")`) and `PREFER_BINARY_SERIALIZATION = true` if it holds bulk data.
2. Register it in `default_asset_types()` (`wutengine_asset_importers/src/generic.rs`), or `weimport` and
   `weassetconv` can't handle it.
3. If it needs a runtime form, implement `FromSerializedAsset` for the runtime type in the crate that owns it.
4. If the editor should show or create it: `editor/wutengine_editor/src/assets/gui/` and `assets/create.rs`.

## Known gaps

- Nothing calls `wutengine_asset_server::init` yet, so `global_asset_server()` panics at runtime. The editor uses its
  own server (`editor/wutengine_editor/src/assets/server.rs`). A game-side loader doesn't exist yet.
