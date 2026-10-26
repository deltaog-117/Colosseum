use super::blast::impact;
use super::health::{Health, Outcome};
use crate::collision::World;
use crate::movement::Player;
use crate::weapon::Detonation;

/// Lets `blast` hurt and push one body: the knockback goes to `player` and
/// the damage to `health`.
///
/// The shooter's own rocket (`is_shooter`) does only a share of its damage
/// but all of its knockback. The dead are untouched, so a corpse is not
/// thrown about, and a body out of reach or walled off takes nothing.
pub fn strike(
    blast: &Detonation,
    world: &World,
    player: &mut Player,
    health: &mut Health,
    is_shooter: bool,
) -> Outcome {
    if !health.is_alive() {
        return Outcome::Ignored;
    }
    let Some(hit) = impact(blast, world, player.bounds()) else {
        return Outcome::Ignored;
    };
    let hit = if is_shooter { hit.to_shooter() } else { hit };
    player.push(hit.impulse());
    health.damage(hit.damage())
}
