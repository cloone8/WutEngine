# A fork of [`puffin`](https://github.com/EmbarkStudios/puffin/commit/7e08a533f9debfb7d051547263d2ab84c666314f)

# Show [`puffin`](https://github.com/EmbarkStudios/puffin/) profiler flamegraph in-game using [`egui`](https://github.com/emilk/egui)

[![Crates.io](https://img.shields.io/crates/v/wutengine_puffin_egui.svg)](https://crates.io/crates/wutengine_puffin_egui)
[![Docs](https://docs.rs/wutengine_puffin_egui/badge.svg)](https://docs.rs/wutengine_puffin_egui)

[`puffin`](https://github.com/EmbarkStudios/puffin/) is an instrumentation profiler where you opt-in to profile parts of your code:

``` rust
fn my_function() {
    puffin::profile_function!();
    if ... {
        puffin::profile_scope!("load_image", image_name);
        ...
    }
}
```

`puffin_egui` allows you to inspect the resulting profile data using [`egui`](https://github.com/emilk/egui) with only one line of code:

``` rust
wutengine_puffin_egui::profiler_window(egui_ctx);
```

<img src="../puffin_egui.gif">

See the [`examples/`](examples/) folder for how to use it with [`eframe`](https://docs.rs/eframe).

To try it out, run `cargo run --release --example eframe`
