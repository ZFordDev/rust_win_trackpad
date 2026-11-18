use super::*;
use std::time::Instant;

fn scroll(source: Source, phase: Phase, y: f64) -> GestureEvent {
    GestureEvent {
        observed_at: Instant::now(),
        source,
        phase,
        gesture: Gesture::Scroll { delta: [0.0, y] },
    }
}

fn frame(ctx: &::egui::Context, time: f64, events: Vec<Event>) -> (Vec2, bool) {
    let mut result = (Vec2::ZERO, false);
    ctx.run_ui(
        ::egui::RawInput {
            time: Some(time),
            events,
            ..Default::default()
        },
        |ui| {
            result = ui.input(|i| (i.smooth_scroll_delta, i.is_scrolling()));
        },
    )
    .drop_without_applying_deltas();
    result
}

#[test]
fn final_move_and_next_frames_mouse_wheel_both_survive() {
    let ctx = ::egui::Context::default();
    let mut input = Input::default();
    let mut events = Vec::new();
    assert!(input.append(
        &[
            scroll(Source::Momentum, Phase::Start, 0.0),
            scroll(Source::Momentum, Phase::Move, -240.0),
            scroll(Source::Momentum, Phase::End, 0.0),
        ],
        2.0,
        Modifiers::NONE,
        &mut events
    ));
    assert_eq!(frame(&ctx, 0.0, events).0, Vec2::new(0.0, -120.0));

    let mut events = vec![wheel(
        Vec2::new(0.0, 3.0),
        TouchPhase::Move,
        MouseWheelSource::Wheel,
        Modifiers::NONE,
    )];
    assert!(!input.append(&[], 2.0, Modifiers::NONE, &mut events));
    assert_eq!(frame(&ctx, 0.01, events).0, Vec2::new(0.0, 3.0));
    assert!(!input.append(&[], 2.0, Modifiers::NONE, &mut Vec::new()));
    println!("egui: final native delta survives same-frame End; next-frame wheel is retained.");
}

#[test]
fn momentum_gap_latches_modifiers_then_new_contact_releases_them() {
    let ctx = ::egui::Context::default();
    let mut input = Input::default();
    let mut events = Vec::new();
    input.append(
        &[
            scroll(Source::Contact, Phase::Start, 0.0),
            scroll(Source::Contact, Phase::Move, -100.0),
        ],
        1.0,
        Modifiers::SHIFT,
        &mut events,
    );
    assert_eq!(frame(&ctx, 0.0, events).0, Vec2::new(-100.0, 0.0));
    let mut events = Vec::new();
    input.append(
        &[scroll(Source::Momentum, Phase::Move, -200.0)],
        1.0,
        Modifiers::NONE,
        &mut events,
    );
    assert!(events.iter().any(|e| matches!(
        e,
        Event::MouseWheel {
            source: MouseWheelSource::Momentum,
            ..
        }
    )));
    assert_eq!(frame(&ctx, 0.7, events).0, Vec2::new(-200.0, 0.0));
    assert_eq!(
        frame(&ctx, 0.71, Vec::new()).0,
        Vec2::ZERO,
        "no extra smoothed travel"
    );

    let mut events = Vec::new();
    input.append(
        &[
            scroll(Source::Momentum, Phase::Cancel, 0.0),
            scroll(Source::Contact, Phase::Start, 0.0),
            scroll(Source::Contact, Phase::Move, 40.0),
            scroll(Source::Contact, Phase::End, 0.0),
        ],
        2.0,
        Modifiers::NONE,
        &mut events,
    );
    assert_eq!(frame(&ctx, 0.72, events).0, Vec2::new(0.0, 20.0));
    let mut events = Vec::new();
    assert!(!input.append(&[], 2.0, Modifiers::NONE, &mut events));
    assert_eq!(frame(&ctx, 0.73, events), (Vec2::ZERO, false));
    println!(
        "egui: momentum gap adds no smoothing; new contact resets modifiers and uses current DPI."
    );
}

#[test]
fn batched_gestures_keep_distance_and_pinch_only_zooms() {
    let ctx = ::egui::Context::default();
    let mut input = Input::default();
    let mut events = Vec::new();
    input.append(
        &[
            scroll(Source::Contact, Phase::Start, 0.0),
            scroll(Source::Contact, Phase::Move, -80.0),
            scroll(Source::Contact, Phase::Move, 40.0),
            scroll(Source::Contact, Phase::End, 0.0),
            scroll(Source::Contact, Phase::Start, 0.0),
            scroll(Source::Contact, Phase::Move, -20.0),
            scroll(Source::Contact, Phase::Cancel, 0.0),
            GestureEvent {
                gesture: Gesture::Pinch { scale: 1.25 },
                ..scroll(Source::Contact, Phase::Move, 0.0)
            },
        ],
        1.0,
        Modifiers::NONE,
        &mut events,
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::Zoom(scale) if *scale == 1.25)));
    assert_eq!(frame(&ctx, 0.0, events).0, Vec2::new(0.0, -60.0));
    input.cancel();
    let mut events = Vec::new();
    assert!(!input.append(&[], 1.0, Modifiers::NONE, &mut events));
    assert_eq!(frame(&ctx, 0.01, events), (Vec2::ZERO, false));
    println!(
        "egui: batched reversals preserve distance, pinch emits Zoom, disable releases input."
    );
}
