# `wutengine.graphics`

- `backend`: The graphics backends to search for, and to use if available. Any (or multiple) of `dx12`, `vulkan`, `metal` and `opengl`
- `debug_level`: The validation level to use when submitting commands to the GPU. Not usually needed, except when manually calling render commands or when debugging the engine renderer. Can be either `unsafe`, `basic`, `debug` or `advanced`
- `ignore_errors`: WutEngine usually panics and crashes when a GPU-level error is encountered, as these are mostly caused by incorrect engine code. This can be disabled if this is set to `false`.
