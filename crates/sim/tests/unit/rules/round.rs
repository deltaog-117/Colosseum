use sim::rules::tuning::{INTERMISSION_SECONDS, ROUND_SECONDS};
use sim::rules::{Phase, Round, Transition};
use sim::tick::TickRate;

const RATE: TickRate = TickRate::DEFAULT;
/// The ticks the round and the intermission come to at 120 Hz.
const ROUND_TICKS: u32 = 21_600;
const INTERMISSION_TICKS: u32 = 960;

fn run(round: &mut Round, ticks: u32) -> Vec<Transition> {
    (0..ticks).filter_map(|_| round.step()).collect()
}

#[test]
fn the_tuning_matches_the_tick_counts_used_here() {
    assert_eq!((ROUND_SECONDS * RATE.hz() as f32) as u32, ROUND_TICKS);
    assert_eq!(
        (INTERMISSION_SECONDS * RATE.hz() as f32) as u32,
        INTERMISSION_TICKS
    );
}

#[test]
fn a_new_round_is_playing_with_the_full_clock() {
    let round = Round::new(RATE);
    assert_eq!(round.phase(), Phase::Playing);
    assert_eq!(round.seconds_left(), ROUND_SECONDS as u32);
    assert_eq!(round.score().frags(), 0);
}

#[test]
fn the_round_ends_on_the_last_tick() {
    let mut round = Round::new(RATE);
    assert!(run(&mut round, ROUND_TICKS - 1).is_empty());
    assert!(round.is_playing());
    assert_eq!(round.step(), Some(Transition::Ended));
    assert_eq!(round.phase(), Phase::Intermission);
}

#[test]
fn the_intermission_ends_in_a_restart_with_a_clean_score() {
    let mut round = Round::new(RATE);
    round.record_kill();
    run(&mut round, ROUND_TICKS);
    assert_eq!(round.score().kills(), 1);
    assert!(run(&mut round, INTERMISSION_TICKS - 1).is_empty());
    assert_eq!(round.step(), Some(Transition::Restarted));
    assert!(round.is_playing());
    assert_eq!(round.score().kills(), 0);
    assert_eq!(round.seconds_left(), ROUND_SECONDS as u32);
}

#[test]
fn the_score_is_frozen_during_the_intermission() {
    let mut round = Round::new(RATE);
    round.record_kill();
    run(&mut round, ROUND_TICKS);
    round.record_kill();
    round.record_death();
    assert_eq!(round.score().kills(), 1);
    assert_eq!(round.score().deaths(), 0);
}

#[test]
fn a_death_counts_while_playing() {
    let mut round = Round::new(RATE);
    round.record_death();
    assert_eq!(round.score().frags(), -1);
}

#[test]
fn the_clock_reads_zero_only_after_the_phase_ends() {
    let mut round = Round::new(RATE);
    run(&mut round, ROUND_TICKS - 1);
    assert_eq!(round.seconds_left(), 1);
}
