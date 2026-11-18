# The egui hookup

This is the small egui 0.37 adapter. I've tested it in the standalone diagnostic
with stock egui, eframe and winit. Scrolling looks good on my test laptop;
the [findings](findings.md) cover what that test tells me so far.

```toml
[target.'cfg(target_os = "windows")'.dependencies]
rust_win_trackpad = { path = "../rust_win_trackpad", features = ["egui"] }
```

The crate requires Rust 1.95, like egui 0.37 itself. Other egui versions aren't
supported by this adapter yet. The native core remains independent of GUI crates.

## Attach once, on the window thread

Get `hwnd` from the live window's `Win32WindowHandle::hwnd`. Keep the guard and
one `rust_win_trackpad::egui::Input::default()` in the per-window app state.
If setup fails, report the error and keep the existing wheel path.

```rust,ignore
let wake_ctx = ctx.clone();
// SAFETY: hwnd belongs to this thread and stays live throughout attachment.
let trackpad = unsafe {
    rust_win_trackpad::Trackpad::attach(hwnd, move || wake_ctx.request_repaint())
}?;
let adapter = rust_win_trackpad::egui::Input::default();
```

The window timer advances Windows input. The wake only asks the app to consume
what arrived. Continuous repaint and the earlier adaptive inertia experiment
aren't needed. Use the window's viewport-specific repaint request for secondary
viewports.

## Before egui consumes each RawInput

Use the current window scale times egui's zoom factor as `pixels_per_point`.
An eframe app can do this conversion in `App::raw_input_hook`. Preserve the
existing event vector, including ordinary mouse wheels.

```rust,ignore
let native = trackpad.take_events();
// Log native here if wanted: timestamps, source, phase and physical movement.
// egui 0.37 puts modifier changes in the event stream.
let modifiers = raw_input.events.iter().fold(ctx.input(|i| i.modifiers), |held, event| {
    match event {
        egui::Event::ModifiersChanged(current) => *current,
        egui::Event::WindowFocused(false) => egui::Modifiers::NONE,
        _ => held,
    }
});
let follow_up = adapter.append(
    &native,
    pixels_per_point,
    modifiers,
    &mut raw_input.events,
);
if follow_up {
    ctx.request_repaint();
}
```

Call `append` even when `native` is empty. egui's End clears the frame's wheel
delta, so the adapter defers an End following movement until one extra frame.
It puts that End before ordinary wheel events so they survive too.

Native movement is already processed by Windows. The adapter establishes a
fresh egui Start each movement batch so an egui timeout doesn't smooth it again.
Original native phases stay in `native` for diagnostics. Modifiers latch across
momentum, with newly pressed modifiers honoured. Pinch maps to `Event::Zoom`.
Bounds, rubber-banding and widget routing remain egui's decisions.

When disabling input, call `trackpad.close()`, drain and append once more, then
drop it. If the guard has already gone, call `adapter.cancel()` and then append
an empty native slice to release egui's gesture state.

Don't enable this alongside the method-2 winit patch, the old native diagnostic
controller or any other touchpad handler. Only one backend should claim contacts.
