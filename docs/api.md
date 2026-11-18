# The small API

The API is deliberately small. The native core has no winit, egui or eframe
dependency. This first version requires Rust 1.95. The optional `egui` feature
targets egui 0.37. I haven't tested an older compiler for core-only use yet.

| Entry point | What it does |
| --- | --- |
| `unsafe Trackpad::attach(hwnd, wake)` | Attach to one live owner-thread window. Returns a guard or `windows::core::Error`. Windows only. |
| `Trackpad::take_events()` | Drain owned `GestureEvent`s once, in callback order. |
| `Trackpad::close()` | Stop native input. Drain once more for cancellation, then drop the guard. |
| `Trackpad::is_closed()` / `last_error()` | Closure state and the last HRESULT, without a logging dependency. |
| Drop the guard | Stop updates, detach the hook and release the native session. |

`hwnd` is a `NonZeroIsize`, matching `raw-window-handle` 0.6's
`Win32WindowHandle::hwnd`. No COM interface appears in the gesture API.

## What comes out

`GestureEvent` contains a monotonic observation time, `Source`, `Phase` and
`Gesture`. The time records when the callback was observed, not a hardware
timestamp.

- `Gesture::Scroll { delta: [x, y] }`: incremental physical pixels. Positive
  values move content right and down.
- `Gesture::Pinch { scale }`: relative multiplier, greater than one zooms in.
  Pinch-anchor translation doesn't also scroll.
- `Phase::{Start, Move, End, Cancel}`: apply movement only on `Move`. Other
  phases carry zero distance or a scale of one.
- `Source::{Contact, Momentum}`: Windows' RUNNING or INERTIA state. Contact
  and momentum share a lifetime. New contact during momentum cancels the old
  gesture; subsequent movement starts another.

Windows supplies momentum. Applications own scale conversion, scroll bounds
and repaint decisions. See the
[egui hookup](egui.md) for the optional adapter.

## Who owns what

- Attach on the live window's owning thread. The guard stays on that thread.
- Keep the window alive through the unsafe attachment call and keep dispatching
  its messages normally. Don't bypass the subclass at destruction.
- Dropping the guard shuts the backend down. Native window destruction does
  too, so dropping the guard afterward is harmless.
- `wake` announces queued input. Just request a frame. Rendering and event
  draining happen later, outside native callbacks and queue locks.
- The native timer pumps input separately from UI repaint requests. There is
  no continuous repaint switch in the fix.
- Only precision-touchpad contacts enter the native path. Windows suppresses
  their compatibility wheel messages. Ordinary mouse wheels retain their route.
  Don't attach this alongside another Direct Manipulation backend.

Attachment checks the owner thread and rejects duplicate hooks. An in-flight
callback retains its state through reentrant teardown. Failed subclass removal
retains closed state for safety, including during window destruction.

The nominal 10 ms timer runs during native input, with up to 25 idle updates
before stopping. Actual delivery can be slower. Native callbacks post a
pointer-free notification; `wake` runs later on the window thread. An unwinding
wake panic disables the backend. A fatal update error closes it too; a rejected
contact only records an error. Drain after closure to cancel a delivered gesture.

## Checks and testing

The focused checks cover native setup, COM queue draining, hook forwarding,
wake delivery, idle shutdown, duplicate rejection and both destruction orders.
Conversion checks cover contact/momentum, reversal, identity resets, cancellation
and pinch. Adapter checks use real egui input processing to check distance,
terminal events, modifiers and a long momentum gap. They don't simulate hardware.

```text
cargo check --offline --no-default-features
cargo test --lib --offline --features egui -- --nocapture
```

I've tested scrolling in the standalone diagnostic with stock egui, eframe
and winit, and it looks good on my Dell Latitude 5520. The
[findings](findings.md) cover that result and what I still need to check.
