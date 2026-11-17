# Where this landed

17 November 2026. A small Windows workaround, still hopefully temporary.

I've tested `rust_win_trackpad` in the existing eframe scrolling diagnostic,
and scrolling looks good on my Dell Latitude 5520. The build uses registry
egui, egui-winit and eframe 0.37.0, plus registry winit 0.30.13. The local crate
supplies Windows Direct Manipulation input, so this works without an egui or
winit fork.

Continuous repaint, adaptive inertia and the old diagnostic native controllers
are disabled in this build. Windows supplies gesture translation and momentum.
The crate's egui adapter feeds the existing ScrollArea through normal input.
It doesn't animate the list offset or predict missing movement.

## What I've checked

| Check | Result |
| --- | --- |
| Scrolling by eye | Looks good in the standalone-crate diagnostic on my Dell Latitude 5520 with its built-in precision touchpad. |
| Release build and resolved dependency graph | Stock egui/eframe/winit, with only this crate loaded locally. |
| Seven focused automated checks | Native session and queue, attachment/teardown, incremental gesture conversion and egui input processing. |
| Final package and API docs | Native package builds on Rust 1.95; optional-egui docs build there with warnings denied. |
| Existing earlier captures | Harness-native and method-2 implementations remain comparison baselines, not measurements of this crate. |

I haven't added a new timing analysis for this crate. The earlier captures
cover the harness-native and method-2 versions. They're useful baselines,
but their numbers shouldn't be presented as measurements of this version.

There are still gaps in the testing. Separate results for boundaries, pinch,
scaling and window lifecycle aren't documented here yet. I also haven't pinned
down the exact cause of the original wheel-delivery issue.

## Why keep this route

- Windows supplies contact and momentum states, avoiding an application flick
  model and its device-specific tuning.
- The core has no winit, egui or eframe dependency. The egui adapter is optional.
- The input timer runs independently of UI repainting. Native callbacks queue
  input; the app drains it later. Ordinary wheel input retains its existing path.
- The session attaches to one window and can be disabled. The original path,
  adaptive baseline and method-2 prototype remain available for comparison.

## The limits

- So far, I've tested this crate on one laptop. Other touchpads, drivers,
  Windows versions, monitor scales and refresh rates still need testing.
- I still need separate hardware results for nested areas, horizontal scrolling,
  focus changes, pinch, boundaries and switching the backend on and off.
- The egui adapter targets 0.37. It handles egui's wheel timeout and same-frame
  End semantics; later egui input changes may need a small adapter update.
- Attaching requires an unsafe call because the app supplies an HWND. The
  [API guide](api.md) explains the thread and window-lifetime rules. If Windows
  won't remove the hook, the crate keeps its closed state alive so callbacks
  can't point into freed memory.
- The app still needs to process Windows messages so the input timer can run.
  The export records callback and consumption times, not Update calls or GPU
  presentation. I haven't explained the earlier prototypes' delivery pauses yet.

## Version 0.1.0

The implementation and local checks are in place for the first experimental
release. The package and API docs build successfully, and hosted docs are
configured to show the Windows API and optional adapter. The build output is
in the investigation's step-6 log.

I'm starting with Rust 1.95 because that's what I've checked, and it's also
egui 0.37's minimum. I hadn't verified the earlier 1.70 declaration, and the
current core dependencies already include crates requiring 1.71. An older
core-only compiler may be possible, but I haven't checked that yet.

I'm keeping the one-laptop limitation in the release notes and will add results
from other setups as they come in.

The app hookup is in [egui.md](egui.md). The workspace diagnostic and capture
instructions are under `test/trackpad-crate/`; they aren't bundled in this crate.
Build logs live under `Investigation Notes/rust-win-trackpad-step-*.txt` in that
workspace. If you're comparing timing, use those logs and the matching gesture
exports alongside the written findings.
