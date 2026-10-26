use glam::Vec3;

use super::brain::{Brain, Target};
use crate::collision::{Aabb, World};
use crate::combat::{self, Health, Outcome};
use crate::movement::{Player, SpawnError};
use crate::rules::{PickupKind, Score, SpawnPoints};
use crate::tick::TickRate;
use crate::weapon::{Detonation, Launcher};

/// A computer-controlled fighter: a body, a launcher, health and a score,
/// steered by a [`Brain`].
///
/// It plays by the player's rules, because it is moved by the same
/// [`Player::step`] from the same kind of command.
#[derive(Debug, Clone)]
pub struct Bot {
    player: Player,
    launcher: Launcher,
    health: Health,
    score: Score,
    brain: Brain,
    /// Where the bot starts each round.
    home: Vec3,
    /// Where the bot last fell, so the respawn can be far from it.
    death_at: Vec3,
    rate: TickRate,
}

impl Bot {
    /// Places a bot with its feet at `home`, loaded and at full health.
    /// `seed` sets how its strafing differs from other bots'.
    ///
    /// # Errors
    ///
    /// Returns [`SpawnError`] if `home` is not finite or the body would
    /// overlap a solid there.
    pub fn try_spawn(
        home: Vec3,
        world: &World,
        rate: TickRate,
        seed: u32,
    ) -> Result<Self, SpawnError> {
        Ok(Self {
            player: Player::try_spawn(home, world, rate)?,
            launcher: Launcher::new(rate),
            health: Health::new(rate),
            score: Score::default(),
            brain: Brain::new(rate, seed),
            home,
            death_at: home,
            rate,
        })
    }

    /// Advances the bot by one tick. A living bot in a round in `playing`
    /// fights `target`, if any; the rest only keep their rockets flying.
    /// Rockets also burst on the bodies in `enemies`, which must leave the
    /// bot's own out.
    pub fn step(
        &mut self,
        world: &World,
        playing: bool,
        target: Option<&Target>,
        enemies: &[Aabb],
    ) {
        let acting = playing && self.health.is_alive();
        let command = acting.then(|| self.brain.decide(&self.player, target, world));
        if let Some(command) = &command {
            self.player.step(command, world);
        }
        self.launcher.step_against(world, enemies);
        if let Some(command) = command
            && command.fire()
            && let Some(aim) = self.player.pose()
        {
            // A denied shot (cooling down, empty) is normal play.
            let _denied = self.launcher.try_fire(&aim);
        }
    }

    /// Lets `blast` hurt and push the bot. `is_shooter` is true for the
    /// bot's own rocket. Counts a death if the blast kills.
    pub fn take_blast(&mut self, blast: &Detonation, world: &World, is_shooter: bool) -> Outcome {
        let outcome = combat::strike(blast, world, &mut self.player, &mut self.health, is_shooter);
        if outcome == Outcome::Killed {
            self.score.record_death();
            self.death_at = self.player.position();
        }
        outcome
    }

    /// Counts a kill for the bot.
    pub fn record_kill(&mut self) {
        self.score.record_kill();
    }

    /// Counts the respawn wait down by one tick and, when it ends, puts the
    /// bot at the spawn point farthest from where it fell with a full
    /// launcher. Call once per tick.
    ///
    /// # Errors
    ///
    /// Returns [`SpawnError`] if that spawn point is blocked; the bot then
    /// stays where it fell.
    pub fn step_health(&mut self, spawns: &SpawnPoints, world: &World) -> Result<(), SpawnError> {
        if !self.health.step() {
            return Ok(());
        }
        self.player = Player::try_spawn(spawns.farthest_from(self.death_at), world, self.rate)?;
        self.launcher = Launcher::new(self.rate);
        Ok(())
    }

    /// Starts a new round: back home, healed, reloaded, with a clean score.
    ///
    /// # Errors
    ///
    /// Returns [`SpawnError`] if home is blocked; the bot then stays put.
    pub fn reset(&mut self, world: &World) -> Result<(), SpawnError> {
        let player = Player::try_spawn(self.home, world, self.rate)?;
        self.player = player;
        self.launcher = Launcher::new(self.rate);
        self.health = Health::new(self.rate);
        self.score = Score::default();
        self.death_at = self.home;
        Ok(())
    }

    /// True if taking a `kind` pickup would change anything.
    pub fn wants(&self, kind: PickupKind) -> bool {
        kind.is_wanted(&self.health, &self.launcher)
    }

    /// Hands the bot what a `kind` pickup holds.
    pub fn receive(&mut self, kind: PickupKind) {
        kind.give(&mut self.health, &mut self.launcher);
    }

    /// The bot's box while it can act, `None` while it is dead or the round
    /// is not in `playing`.
    pub fn active_body(&self, playing: bool) -> Option<Aabb> {
        (playing && self.health.is_alive()).then(|| *self.player.bounds())
    }

    /// What other fighters aim at, or `None` while the bot is dead or its
    /// state is not finite.
    pub fn as_target(&self) -> Option<Target> {
        self.health
            .is_alive()
            .then(|| Target::try_new(self.player.bounds().center(), self.player.velocity()))
            .flatten()
    }

    /// The body.
    pub fn player(&self) -> &Player {
        &self.player
    }

    /// The launcher and the rockets it has in the air.
    pub fn launcher(&self) -> &Launcher {
        &self.launcher
    }

    /// Hit points and the respawn wait.
    pub fn health(&self) -> &Health {
        &self.health
    }

    /// Kills and deaths this round.
    pub fn score(&self) -> &Score {
        &self.score
    }
}
