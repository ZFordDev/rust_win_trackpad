# The small API we're aiming for

This is the plan, not a working public API yet. The private session, callback
queue and window attachment are in. Public gesture conversion comes next.

| Entry point | What it does |
| --- | --- |
| `Trackpad::attach(window, wake)` | Attach to one live Windows window. Return a guard or an initialization error. |
| `Trackpad::take_events()` | Drain owned gesture events for the app to consume once. |
| Drop the guard | Stop updates, detach the hook and release the native session. |

Those names describe the intended shape. Exact types land with the code.
There are no placeholder functions pretending to do the job.

## What comes out

Scroll movement in physical pixels, pinch scale changes and gesture phases.
Windows supplies momentum; we don't invent a flick curve. Contact versus
inertia stays available for diagnostics and the later adapter.

The native core won't depend on winit, egui or eframe. Applications handle
their own scale conversion, scroll bounds and repaint decisions. The optional
egui adapter will handle its input conversion separately.

## Who owns what

- Attach on the live window's owning thread. The guard stays on that thread.
- Keep the guard while input is wanted. Dropping it shuts the backend down.
- Native window destruction must also shut the backend down. Dropping the
  guard afterward must be harmless.
- `wake` announces queued input. Rendering happens later in the app's normal
  frame, outside native callbacks and queue locks.
- The native timer pumps input separately from UI repaint requests.
- Only precision-touchpad contacts enter the native path. Ordinary mouse
  wheels keep their existing route; failed setup leaves that route available.

The window attachment now checks the owner thread and rejects a second hook
on the same window. Dropping the guard or destroying the window stops input.
An in-flight callback retains its state through reentrant teardown. Normal
detachment releases the hook's reference. Failed removal retains the closed
state for safety, including if removal fails during window destruction.

The nominal 10 ms timer runs during native input, with up to 25 idle updates
before stopping. Actual delivery can be slower. Native callbacks post a
pointer-free notification. `wake` runs later on the window thread and should
just request a frame. An unwinding wake panic disables the backend.

## Enough diagnostics to see what's happening

Later stages will add opt-in attachment status, native phases and delivery
timing. The existing investigation captures remain the comparison baseline.
The private queue records native status and six-value transforms with a
monotonic observation time. Draining takes each sample once. It doesn't call
application code or render from a COM callback.

The latest focused check covers hook forwarding, wake delivery, idle timer
shutdown, duplicate rejection, reattachment and both destruction orders.
The earlier session check covers real COM callbacks and one-time draining.
Scrolling gets its visual check when the public API reaches the diagnostic.

```text
cargo test --lib --offline attachment::tests::hook_pump_and_both_destruction_orders -- --exact --nocapture
```

## What's left

Three planned stages: public gesture conversion plus the egui adapter, then
the diagnostic's visual test, then a short results and release-prep pass.
