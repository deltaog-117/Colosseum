use super::score::Score;
use super::tuning::{INTERMISSION_SECONDS, ROUND_SECONDS};
use crate::tick::TickRate;

/// Which part of the round clock is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Bodies move and score.
    Playing,
    /// The round is over and the final score is up.
    Intermission,
}

/// A change of phase, reported on the tick it happens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    /// Time ran out; the score is now final.
    Ended,
    /// The intermission is over; a new round began with a clean score.
    Restarted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Ticks until time is up, always positive.
    Playing(u32),
    /// Ticks until the next round, always positive.
    Intermission(u32),
}

/// The round clock and the score it keeps.
///
/// Rounds repeat: playing, then an intermission, then playing again with a
/// clean score. The score only changes while playing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Round {
    state: State,
    score: Score,
    round_ticks: u32,
    intermission_ticks: u32,
    hz: u32,
}

impl Round {
    /// A round that has just started.
    pub fn new(rate: TickRate) -> Self {
        let ticks = |seconds: f32| ((seconds * rate.hz() as f32).ceil() as u32).max(1);
        let round_ticks = ticks(ROUND_SECONDS);
        Self {
            state: State::Playing(round_ticks),
            score: Score::default(),
            round_ticks,
            intermission_ticks: ticks(INTERMISSION_SECONDS),
            hz: rate.hz(),
        }
    }

    /// Counts the clock down by one tick. Call once per tick.
    pub fn step(&mut self) -> Option<Transition> {
        match self.state {
            State::Playing(1) => {
                self.state = State::Intermission(self.intermission_ticks);
                Some(Transition::Ended)
            }
            State::Playing(ticks) => {
                self.state = State::Playing(ticks - 1);
                None
            }
            State::Intermission(1) => {
                self.state = State::Playing(self.round_ticks);
                self.score = Score::default();
                Some(Transition::Restarted)
            }
            State::Intermission(ticks) => {
                self.state = State::Intermission(ticks - 1);
                None
            }
        }
    }

    /// Counts a kill, unless the round is over.
    pub fn record_kill(&mut self) {
        if self.is_playing() {
            self.score.record_kill();
        }
    }

    /// Counts a death, unless the round is over.
    pub fn record_death(&mut self) {
        if self.is_playing() {
            self.score.record_death();
        }
    }

    /// The running score, or the final one during the intermission.
    pub fn score(&self) -> &Score {
        &self.score
    }

    /// Which part of the clock is running.
    pub fn phase(&self) -> Phase {
        match self.state {
            State::Playing(_) => Phase::Playing,
            State::Intermission(_) => Phase::Intermission,
        }
    }

    /// True while bodies may move and score.
    pub fn is_playing(&self) -> bool {
        self.phase() == Phase::Playing
    }

    /// Whole seconds left in the current phase, rounded up so the clock
    /// reads zero only when the phase has ended.
    pub fn seconds_left(&self) -> u32 {
        let (State::Playing(ticks) | State::Intermission(ticks)) = self.state;
        ticks.div_ceil(self.hz)
    }
}
