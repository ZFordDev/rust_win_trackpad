# The small API we're aiming for

This is the plan, not a working public API yet. The private native session
can open, update and close. Window attachment and input delivery come next.

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

The public lifetime rules get checked as the window attachment lands.
The private session already rejects calls after close and balances COM
initialization on its owning thread.

## Enough diagnostics to see what's happening

Later stages will add opt-in attachment status, native phases and delivery
timing. The existing investigation captures remain the comparison baseline.
The session now has one hidden-window smoke test. It exercises real Windows
setup, a manual update and repeatable cleanup; it doesn't test scrolling.

```text
cargo test --lib --offline -- --nocapture
```
