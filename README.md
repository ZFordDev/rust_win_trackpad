# rust_win_trackpad

A temporary Rust crate to help fix precision touchpad scrolling on Windows.

## Where things stand

The native session, window hook and timer pump are in, along with the small
gesture API and optional egui 0.37 adapter. Windows handles the momentum.

The [small API guide](docs/api.md) covers attachment, input delivery and cleanup.
The [hookup notes](docs/egui.md) show the two places an app needs to call it.
I've tested the standalone diagnostic with stock egui, eframe and winit, and
the initial results look promising on my Dell Latitude 5520 with continuous
repaint OFF. I'm still testing the behaviour, and other devices need testing too.

The short [findings and release notes](docs/findings.md) cover what passed and
what's still open. This first version requires Rust 1.95.

> [!IMPORTANT]
> This is hopefully a temporary repository addressing one tiny issue in a very specific place.
>
> I'm preparing fixes for **winit** and **egui**, but they're still being tested.
> Once I'm happy with the fix, I'll open the upstream pull requests. Hopefully
> it can eventually land there and make this crate unnecessary.

## Why does this exist?

The basic problem is that `winit` doesn't reliably expose everything needed to properly handle Windows precision touchpad scrolling.

It's such a niche problem that creating an entire crate for it seems a little ridiculous, but here I am, neck deep into investigating and fixing one tiny scrolling issue...

The original investigation started with Qt's `qtbase` implementation. Testing was then carried out using PySide6, which actually uses a different Qt version from the source I was inspecting.

Interestingly, newer Qt implementations have introduced a dedicated precision touchpad handling section alongside their legacy scrolling implementation. That suggests this isn't exactly a new problem.

## Why not just fix egui?

Good question!

Technically, we could implement the workaround directly in `egui` and `eframe`, but doing so would introduce a considerable amount of Windows-specific code.

I didn't particularly want to dump all of that on Emil and the egui maintainers.

Instead, I went further upstream to `winit`, which already has a surprising amount of the infrastructure needed to implement the fix.

Almost like that's what it was designed to do. Haha.

The problem is that upstream changes aren't guaranteed to be accepted.

`winit` makes it clear in its contributing guidelines that larger changes can face considerable scrutiny, which is fair enough. Maintaining a cross-platform windowing library is no small task.

But I also need this issue fixed across several of my own applications, regardless of whether a future upstream PR is accepted.

**And so here we are.**

An entire crate to address one very specific Windows scrolling problem, with the hope that one day somebody upstream will take the workload off my hands.

## The long-term plan

Ideally, this repository becomes obsolete.

The preferred solution is proper support upstream in `winit`, followed by the necessary integration in `egui`.

Until then, this crate exists as a practical workaround for applications that need better Windows precision touchpad handling.

I don't intend to turn this into a massive input-handling framework. The goal is to solve the problem, keep the implementation focused, and eventually retire it if upstream support makes it redundant.

## Issues and feedback

If you encounter a problem, please open an issue.

I'll investigate what I can, and where appropriate, pass findings or potential improvements upstream to `winit` and `egui` as that work progresses.

No promises that every device will behave perfectly, but I'll do what I can.

---
