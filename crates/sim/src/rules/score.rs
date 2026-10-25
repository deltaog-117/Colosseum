/// Kills and deaths of one player in a round.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Score {
    kills: u32,
    deaths: u32,
}

impl Score {
    /// Counts one kill.
    pub fn record_kill(&mut self) {
        self.kills = self.kills.saturating_add(1);
    }

    /// Counts one death.
    pub fn record_death(&mut self) {
        self.deaths = self.deaths.saturating_add(1);
    }

    /// Kills so far.
    pub fn kills(&self) -> u32 {
        self.kills
    }

    /// Deaths so far.
    pub fn deaths(&self) -> u32 {
        self.deaths
    }

    /// Kills minus deaths; negative when the player dies more than they kill.
    pub fn frags(&self) -> i64 {
        i64::from(self.kills) - i64::from(self.deaths)
    }
}
