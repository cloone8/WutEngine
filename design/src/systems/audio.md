# Audio

The audio API, the mixer's place on desktop and the web, sounds, buses and volume, positional sound, time and pause,
and formats. The mixer's internals are decided when built.

## Our own mixer, one batch per frame, in an audio worklet on the web

**The engine's own mixer runs on the audio output's thread on desktop and on the browser's audio thread on the web,**
as a small separate wasm module in an AudioWorklet.
Why: mixing on the main thread on a single-threaded web build starves the output whenever a frame is late, and
letting the browser mix would mean a second implementation of every feature with its own limits, so games would sound
different on the web. One mixer gives the same sound everywhere and no glitch when a frame is late.

- **Two halves.** The **game side** (main thread, part of the engine): sources, playbacks, buses, limits, positions,
  each play's variation (clip, pitch, volume), and each sound's time. The **mixer** (its own crate with no engine
  dependency, so it builds as a separate small wasm module): voices, panning, bus mixing, output.
- **One batch per frame.** The game side sends the frame's changes as one batch of plain-data messages in render
  preparation, so a sound's start and the listener's move in the same frame apply together; on the web through the
  worklet's message port. A sound's data is sent when the sound is loaded and dropped when it's freed; a playing sound
  keeps its asset loaded until it ends.
- **The game side keeps each sound's time,** from the clip length stored at import and the pitch, without asking the
  mixer. It advances by the **unclamped real-time delta,** not the clamped frame delta, because the mixer keeps
  playing during a hitch. It stops only for a paused bus, the development tools' pause and backgrounded. So whether a
  sound has finished, its playback position and `finished` are the same in a headless build, while the web waits for
  a click, and for the silence placeholder; the game never waits on the mixer.
- **Plays from parallel Rust systems are queued and applied in a fixed order** (system registration order, then call
  order), so instance limits pick the same sounds on every platform.
- **Stand-ins:** cpal for desktop output; decoders from crates. The mixer itself is in-house from the start, since a
  crate mixer would bring back main-thread mixing on the web.
- **Costs:** a second wasm module the loader library loads, and sound data copied into the worklet's memory. Kept
  possible: with wasm threads, the mixer sharing memory instead of copying, behind the same messages.

## Sources and one-shot calls; playbacks that never throw

- **An `AudioSource` component** for what belongs to an entity: an engine hum, a radio, ambience. It plays a sound,
  with autoplay, loop, volume, pitch and bus; it follows its entity; it **stops with a short fade when the component or
  its entity goes away.** **The component is its own control:** `play()`, `stop()`, its fields, `playing`, and a
  `finished` signal on the component. It gives no playback object, so a source has one control surface.
- **A non-positional `AudioSource` may sit on an entity without a transform,** such as a global entity, which survives
  a scene switch: music that spans levels ([Scenes](../world/scenes.md)). A positional source needs a transform.
- **A plain function for one-shots:** `audio.play(sound, { at: position })` or `{ follow: entity }`, with optional
  volume, pitch and bus. **It doesn't go through `this.`, so it doesn't end with a component:** an explosion or a death
  cry plays to the end. A followed entity that disappears leaves the sound at its last position. Positions held on the
  game side are `WorldPos` and shift with the origin ([Game size and precision](../goals/game_size.md)).
- **At a scene switch** non-positional one-shots play on (a "Start game" click, a jingle); positional ones stop with a
  short fade, since their coordinates belong to the old level.
- **`audio.play` returns a playback.** Calls on a playback whose sound has ended **do nothing** (`playing` is
  `false`). Why: a sound ends by itself at a moment code doesn't choose, so throwing would force a check before every
  call; an entity is destroyed by code, so a stale entity handle throwing is a bug worth reporting.
- **`playback.finished` is an engine promise,** settled at the settle point ([Script API](../scripting/api.md)), and
  **already settled once the sound has ended,** so `await this.wait(line.finished)` after the sound ended doesn't hang.
  It settles however the sound ends: played out, `stop()`, stopped or refused by an instance limit, or replaced by
  silence. The same holds for an `AudioSource`'s `finished` signal.
- **Rust uses the same functions and playback objects;** it reads `playing` instead of awaiting.

## One `Sound` asset type, with variations

- **An imported audio file is a `Sound`** with one clip; its import settings are that `Sound`'s settings. **A designed
  `Sound`** lists other sounds' clips and how to pick (random without repeating the last, sequential, random), volume
  and pitch ranges, default bus, positional or not, distance settings and an instance limit. Footsteps are one
  `Sound` with six clips and a little pitch variation. Why: code plays designed units, not files, and one type fits
  every slot (dragging a file onto a source just plays it).
- **Positional or non-positional:** a non-positional sound (music, UI clicks) ignores position; a positional sound
  played without a position plays at the listener. An imported file's default is positional.
- **Which bus:** the `audio.play` option, else the `AudioSource`'s field, else the `Sound`'s default; an imported
  file's default is `Effects`.
- **Kept possible:** parameters driving a sound, interactive music, a tail clip played when a loop stops.

## Buses, player volume, limits

- **One bus tree per project,** a project asset: `Master` with, by default, `Music`, `Effects`, `Dialogue` and `UI`.
  A bus has a mix volume, mute and "pauses with the game". Kept possible: effects on buses, ducking, snapshots.
- **A bus has a UUID identity and a readable `Name`.** Sounds and sources store the UUID, so renaming a bus never
  reroutes anything; code may look a bus up by name (`audio.bus("Music")`), unknown names logged in development
  builds.
- **Player volume is configuration:** the engine's `audio` section has one player field, **a map from bus `Name` to
  the player's volume** (readable in the settings file and at the console, such as `audio.volume.Music`; renaming a
  bus forgets its saved value) ([Configuration](../core/configuration.md)). An options menu sets it, and it is stored
  in the player settings with no save code; the game chooses which buses its menu shows, and a template options menu
  shows the default ones. **What is heard is the bus's mix volume (which code may fade) times the player's volume,** so
  a scripted music fade never reaches the settings file. The slider value is linear in perceived loudness. Why a map:
  config fields are Rust struct fields, so a bus added in the editor can't be one.
- **Instance limits per `Sound`:** at most N playing, then stop the oldest (the default) or don't start the new one;
  decided on the game side, so the same everywhere. How many voices the mixer renders, and which quiet or distant
  sounds it skips while their time runs on, is internal.

## Positional sound

- **An `AudioListener` component,** on the camera in the templates. One is active; with two, the first in hierarchy
  order wins and a warning is logged. With none, positional sounds are silent and a warning is logged once. Kept
  possible: several listeners (split screen).
- **Positions:** in render preparation, sources and the listener read their **displayed (interpolated) pose**, so
  what is heard matches what is seen ([Time and the frame](../core/frame.md)). The game side turns them into
  listener-relative f32 positions before sending, so the mixer never sees `WorldPos`.
- **Distance:** minimum and maximum distance with a curve per `Sound`; past the maximum a sound isn't mixed, but its
  time runs on. Which sources are in range is audio's own structure.
- **Panning:** stereo, equal-power, first. Kept possible: HRTF, Doppler, reverb regions, obstruction through physics
  ray queries, surround output.

## Time: pause by bus, real-time playback

- **Pause:** while the game is paused, buses marked "pauses with the game" pause (default: `Effects` and `Dialogue`)
  and the others play on (`Music`, `UI`), so a pause menu's clicks are heard. One rule in the bus asset, not a flag per
  source.
  - The setting is inherited down the bus tree; `Master` never pauses.
  - A sound started on a paused bus while the game is paused starts paused.
  - **An entity's "runs while paused" has no effect on sound:** a pause menu's `AudioSource` plays only if its bus
    doesn't pause. Why: music on a scene entity plays on through a pause menu.
  - The development tools' pause freezes all sound.
- **Time scale:** sounds play in real time and don't follow the time scale. Kept possible: a per-bus "pitch follows
  time scale" for slow motion.
- **Kept possible: the audio clock:** reading how far the output has played (rhythm, music sync) and starting a sound
  at an exact audio time.
- **Focus loss:** nothing happens; "mute when unfocused" is a template option.
- **Backgrounded:** the engine suspends audio output and resumes it when foregrounded. Why: the game's frames stop
  there, and on the web the worklet would otherwise loop the last sounds forever. On the web the loader suspends and
  resumes the audio context itself on page visibility changes.

## Middleware: not integrated

**No middleware layer in the engine.** Why: Wwise has no public web platform, FMOD's web build is an Emscripten
library, both have budget-bounded free tiers, and the first game needs neither. Kept possible: a third-party
integration that brings its own components.

## The web: audio starts at the first click, tap or key

Browsers start audio suspended until a user gesture.
- **The loader library resumes audio on the first click, tap or key press,** the events browsers count as user
  activation. Its own listeners stay until audio actually runs, and work whatever the engine's input layer does.
- **Until then the game side runs as usual:** looping and long sounds are heard from their current position once audio
  starts, one-shots that ended meanwhile are never heard, and `finished` settles on time.
- **The default web template has a "click to start" option, off by default.** Why: most games start with a menu that
  needs a click anyway ([Building and shipping](../editor/shipping.md)).

## Formats and loading

- **Runtime formats are cargo features: WAV and Ogg Vorbis on by default,** MP3 and FLAC opt-in. Why: royalty-free,
  and each decoder costs only tens of KB on the web. Kept possible: Opus, once a mature pure-Rust decoder exists.
- **Our own decoders on the web too,** for the same sound everywhere. Kept possible: the browser's decoder.
- **Each clip's length** (frames and sample rate) is stored at import, so the game side keeps sound time without
  decoding, also in headless builds.
- **Two load modes, an import setting per clip:** decoded at load (short sounds), or **kept compressed in memory and
  decoded while playing** (music, long ambience), the default above some length. No sound banks: sounds load with
  their scenes like any asset ([Assets and project files](../world/assets.md)). Kept possible: streaming from disk,
  re-encoding at import.

## Editor

The `AudioSource` and `AudioListener` components and the bus asset are edited in the inspector; `Sound` assets get a
small editor with a play button.
