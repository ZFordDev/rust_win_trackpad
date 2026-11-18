# Changelog

## 0.1.0

- Added Windows precision-touchpad input using Direct Manipulation and
  Windows-managed momentum.
- Added physical-pixel scroll events, relative pinch scales, native phases
  and callback observation times.
- Added the optional egui 0.37 adapter, with input-driven repainting and no
  custom inertia controller. Requires Rust 1.95.

Scrolling looks good in the diagnostic on my Dell Latitude 5520. This is the
first experimental release; other devices and interaction cases still need
testing. See [the findings](docs/findings.md) for the details.
