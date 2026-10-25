use proptest::prelude::*;
use sim::rules::{Phase, Round, Transition};
use sim::tick::TickRate;

/// Per tick: a kill, a death, or nothing.
fn plays() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(0u8..3, 1..60_000)
}

proptest! {
    #[test]
    fn transitions_alternate_and_the_score_only_moves_while_playing(play in plays()) {
        let mut round = Round::new(TickRate::DEFAULT);
        let mut last: Option<Transition> = None;
        for action in play {
            let before = *round.score();
            let was_playing = round.phase() == Phase::Playing;
            match action {
                1 => round.record_kill(),
                2 => round.record_death(),
                _ => {}
            }
            if !was_playing {
                prop_assert_eq!(*round.score(), before);
            }
            if let Some(transition) = round.step() {
                prop_assert_ne!(Some(transition), last);
                last = Some(transition);
                if transition == Transition::Restarted {
                    prop_assert_eq!(round.score().frags(), 0);
                }
            }
            prop_assert!(round.seconds_left() >= 1);
        }
    }
}
