# Save games and storage

How games save progress, how writes survive crashes, where per-user files live on each platform, and what is kept
possible for cloud and platform saves. Player settings are configuration, not saves
([Configuration](../core/configuration.md)).

## The engine stores; the game decides what a save contains

- **A save is game data the game chooses** (progress, inventory, which doors are open): plain data in TypeScript, a
  serde type in Rust. Not a snapshot of the world. Why: script state can't be saved (a running `await` has no
  serialized form), and a snapshot would need rules for every script change between saving and loading.
- **Saves hold game data, not components or live references:** assets are stored as their UUIDs (`AssetRef`) and
  loaded by game code, entities as IDs.
- **Restoring is game code:** load the scene, then apply the saved data. A game stores "this door" by its scene and ID
  (a chain of IDs inside instances, [Scenes](scenes.md)); positions are saved as absolute positions
  ([Game size and precision](../goals/game_size.md)).
- **A template "save system" script** in the template project shows the pattern: collect state from objects tagged
  `Saved`, write, load, apply; it keeps a `version` field in its data and shows migrating an older one, and it asks for
  persistent storage on the web.
- Kept possible: an engine helper that saves a scene's changed entities as overrides against the scene file.

## `saves`: a small async API for named saves

- **`saves`** is an engine service with plain functions: `write(name, data)`, `read(name)`, `list()`, `delete(name)`,
  each returning a promise (scripts) or a `Task` (Rust), never blocking. The engine maps names to files or browser
  records. In a script: `await this.wait(saves.write("slot1", data))`; `read` gives the data, or `null` when there is
  none.
- **A save name** is the game's ("slot1", "autosave"): letters, digits, `-` and `_`, with a length limit, since it
  becomes a file name. In Rust a `SaveName` type checked when made; in scripts a string checked by the call, an invalid
  name being misuse ([Error handling](../core/errors.md)).
- **A write, once started, always completes:** dropping its task, or the script that awaited it going away ("save and
  quit to menu"), only drops the result. A stated exception to "dropping a task cancels it"
  ([Jobs and async](../core/jobs.md)); the quit flush step waits for it.
- **The format is JSON,** tolerant of added and removed fields by name and readable when debugging a player's report.
  The engine adds a small header with what it needs itself (an engine format number and a timestamp). **The save's
  version is the game's,** a field in its own data, migrated by its own code.
- **No second key-value store** for player preferences, and **no general virtual file system** for game code: games
  reach assets through the asset system and saves through `saves`; Rust code on desktop can still use `std::fs`.

## Writes that survive a crash

- **The atomic write,** shared with the editor's save: each write goes to a temporary file, is flushed to the disk,
  then replaces the old file, using Rust's standard library (`File::sync_all`, `fs::rename`). On Linux and macOS the
  engine also syncs the folder, so the rename itself survives a power cut. A crash leaves the old file or the new one,
  never half of each.
- **For saves, a backup:** the previous version is kept. When the main file is missing or doesn't parse or validate,
  `read` falls back to the backup and logs it. **No checksum.** Why: a truncated file already fails to parse,
  write-then-replace prevents half-written files, and a checksum would reject saves edited by hand, which players and
  developers do.
- **Timing:** the quit flush step waits for writes in progress; backgrounded is where the game starts its save, early
  enough to finish; a write on a fatal error runs under the hook's timeout ([Lifecycle and
  memory](../core/lifecycle.md)).

## Where files live

Per user, under `<Org>/<Game>` from the build, in the `dirs` crate's folders; no per-platform folder code of the
engine's own. The web is the exception.

| | Function | Linux | macOS | Windows |
|---|---|---|---|---|
| Saves (`saves/`) | `dirs::data_dir` | `$XDG_DATA_HOME` | `~/Library/Application Support` | `AppData\Roaming` |
| Player settings | `dirs::config_local_dir` | `$XDG_CONFIG_HOME` | `~/Library/Application Support` | `AppData\Local` |
| Logs (`logs/`) | `dirs::data_local_dir` | `$XDG_DATA_HOME` | `~/Library/Application Support` | `AppData\Local` |

- **Saves roam on Windows:** they follow the player between machines, and Steam Auto-Cloud can sync from the folder.
- **Settings are local:** they hold machine-specific values such as video quality.
- **Logs:** one function on every platform, simpler than a fallback. `latest.log` and `previous.log`, rotated at
  start-up.
- **Saves and logs get their own subfolders,** since several rows share a folder on macOS and Linux. Editor
  preferences follow the player settings row.
- **A folder that can't be found** (no home folder) is logged (to stderr when it's the log folder): saves fail with an
  error, settings stay in memory, no log file is written.
- **Play mode keeps its saves, player settings and logs in the project's ignored per-user folder,** so testing never
  touches an installed copy's files; the editor has a "clear saves" command.
- **The web:** saves in IndexedDB, settings in `localStorage`, logs in the browser console; the database and keys are
  named from `<Org>/<Game>` too.
- Android, which `dirs` treats as Linux, will be a second exception.

## The web: IndexedDB, persistence on request

- **Saves go to IndexedDB, one transaction per write** (header and data together), so a write is all or nothing. The
  API is async anyway, so there is no preloaded file system and no write-back step to forget. No backup on the web.
- **`saves.requestPersistence()`,** called by the game, asks the browser for persistent storage, so it doesn't evict
  the saves under storage pressure. Why the game calls it: some browsers show the player a prompt, so the engine never
  asks on its own. The template save script calls it; on desktop it does nothing.
- **Documented, not solved:** browsers may delete the data of sites not visited for a while; embedding sites partition
  storage; IndexedDB writes aren't guaranteed to finish if the tab closes, which is why the game starts its save when
  backgrounded.
- Kept possible: "export and import save" as a file download.

## Cloud and platform saves: kept possible

- **Steam Auto-Cloud works without code** with the folders above; a Steamworks integration is kept possible.
- **The `saves` API already has the shape platform save systems have** (named entries, async, atomic per write), so
  they can sit behind it later. A platform user is added to the API when a platform needs it.

## A save is untrusted input

- **Reading a save is a trust boundary:** plain JSON data, never class instances or code; the header's format number
  checked; a size limit; errors returned rather than panics.
- **No encryption or signing.** Why: a player can edit their own saves on PC anyway. Kept possible for games that need
  it.
