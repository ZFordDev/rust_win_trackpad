use super::*;

fn push(converter: &mut Converter, event: NativeEvent, events: &mut Vec<GestureEvent>) {
    converter.append(
        Sample {
            observed_at: Instant::now(),
            event,
        },
        events,
    );
}

fn translation(y: f32) -> NativeEvent {
    NativeEvent::Transform([1.0, 0.0, 0.0, 1.0, 0.0, y])
}

#[test]
fn contact_momentum_reversal_and_identity_reset_keep_distance() {
    let mut converter = Converter::default();
    let mut events = Vec::new();
    for event in [
        NativeEvent::Status(DIRECTMANIPULATION_RUNNING),
        translation(-80.0),
        NativeEvent::Status(DIRECTMANIPULATION_INERTIA),
        translation(-240.0),
        NativeEvent::Status(DIRECTMANIPULATION_RUNNING),
        translation(-200.0),
        NativeEvent::Status(DIRECTMANIPULATION_READY),
        translation(0.0), // The native reset is not user movement.
        NativeEvent::Status(DIRECTMANIPULATION_RUNNING),
        translation(-20.0),
        NativeEvent::Status(DIRECTMANIPULATION_READY),
    ] {
        push(&mut converter, event, &mut events);
    }
    let moves: Vec<_> = events.iter().filter(|e| e.phase == Phase::Move).collect();
    assert_eq!(
        moves
            .iter()
            .map(|e| (e.source, e.gesture))
            .collect::<Vec<_>>(),
        vec![
            (
                Source::Contact,
                Gesture::Scroll {
                    delta: [0.0, -80.0]
                }
            ),
            (
                Source::Momentum,
                Gesture::Scroll {
                    delta: [0.0, -160.0]
                }
            ),
            (Source::Contact, Gesture::Scroll { delta: [0.0, 40.0] }),
            (
                Source::Contact,
                Gesture::Scroll {
                    delta: [0.0, -20.0]
                }
            ),
        ]
    );
    assert_eq!(
        events.iter().map(|e| e.phase).collect::<Vec<_>>(),
        vec![
            Phase::Start,
            Phase::Move,
            Phase::Move,
            Phase::Cancel,
            Phase::Start,
            Phase::Move,
            Phase::End,
            Phase::Start,
            Phase::Move,
            Phase::End,
        ]
    );
    assert!(events
        .windows(2)
        .all(|pair| pair[0].observed_at <= pair[1].observed_at));
    println!("Conversion: native momentum, reversal and reset preserve incremental distance.");
}

#[test]
fn pinch_excludes_anchor_motion_and_close_cancels_once() {
    let mut converter = Converter::default();
    let mut events = Vec::new();
    for event in [
        NativeEvent::Status(DIRECTMANIPULATION_RUNNING),
        translation(-10.0),
        NativeEvent::Transform([f32::NAN; 6]), // Must not poison the next delta.
        NativeEvent::Transform([2.0, 0.0, 0.0, 2.0, -100.0, -200.0]),
        NativeEvent::Transform([3.0, 0.0, 0.0, 3.0, -400.0, -600.0]),
        NativeEvent::Transform([3.0, 0.0, 0.0, 3.0, -500.0, -700.0]),
    ] {
        push(&mut converter, event, &mut events);
    }
    assert_eq!(
        events
            .iter()
            .filter(|e| e.phase == Phase::Move)
            .map(|e| e.gesture)
            .collect::<Vec<_>>(),
        vec![
            Gesture::Scroll {
                delta: [0.0, -10.0]
            },
            Gesture::Pinch { scale: 2.0 },
            Gesture::Pinch { scale: 1.5 },
        ]
    );
    converter.cancel(Instant::now(), &mut events);
    let count = events.len();
    converter.cancel(Instant::now(), &mut events);
    push(&mut converter, translation(-999.0), &mut events);
    assert_eq!(events.len(), count);
    assert_eq!(events.last().unwrap().phase, Phase::Cancel);
    println!(
        "Conversion: pinch anchor excluded, invalid matrix ignored, cancellation delivered once."
    );
}
