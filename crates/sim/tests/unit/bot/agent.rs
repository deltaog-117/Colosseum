use glam::Vec3;
use sim::bot::{Bot, Target};
use sim::collision::{Aabb, World};
use sim::combat::tuning::MAX_HEALTH;
use sim::combat::{Outcome, tuning::RESPAWN_SECONDS};
use sim::movement::tuning::EYE_HEIGHT;
use sim::movement::{Pose, SpawnError};
use sim::rules::{PickupKind, SpawnPoints};
use sim::tick::TickRate;
use sim::weapon::tuning::START_AMMO;
use sim::weapon::{Detonation, Launcher};

const RATE: TickRate = TickRate::DEFAULT;

fn aabb(min: Vec3, max: Vec3) -> Aabb {
    match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    }
}

/// A big floor, and a wall 1.5 m behind the bot's start, so a rocket that
/// bursts on it is well within reach of a body at the origin.
fn world() -> World {
    World::new(vec![
        aabb(
            Vec3::new(-100.0, -1.0, -100.0),
            Vec3::new(100.0, 0.0, 100.0),
        ),
        aabb(Vec3::new(-30.0, 0.0, 1.5), Vec3::new(30.0, 10.0, 2.5)),
    ])
}

fn spawn(world: &World) -> Bot {
    match Bot::try_spawn(Vec3::ZERO, world, RATE, 1) {
        Ok(bot) => bot,
        Err(err) => panic!("fixture spawn is free: {err}"),
    }
}

fn mark(z: f32) -> Target {
    match Target::try_new(Vec3::new(0.0, EYE_HEIGHT, z), Vec3::ZERO) {
        Some(target) => target,
        None => panic!("fixture target is finite"),
    }
}

/// A blast that lands on a body standing at the origin: a rocket fired
/// from z = -10 along +Z into the wall just behind it.
fn blast_on_origin(world: &World) -> Detonation {
    let Some(aim) = Pose::try_new(Vec3::new(0.0, EYE_HEIGHT, -10.0), std::f32::consts::PI, 0.0)
    else {
        panic!("fixture pose is finite");
    };
    let mut launcher = Launcher::new(RATE);
    if let Err(denied) = launcher.try_fire(&aim) {
        panic!("the launcher should fire: {denied}");
    }
    for _ in 0..600 {
        launcher.step(world);
        if let Some(detonation) = launcher.detonations().first() {
            return *detonation;
        }
    }
    panic!("the rocket never detonated");
}

/// Repeats the blast until it kills; bounded, so a miss fails instead of hanging.
fn kill(bot: &mut Bot, blast: &Detonation, world: &World) {
    for _ in 0..10 {
        if bot.take_blast(blast, world, false) == Outcome::Killed {
            return;
        }
    }
    panic!("ten blasts did not kill the bot");
}

fn spawns() -> SpawnPoints {
    match SpawnPoints::try_new(vec![Vec3::ZERO, Vec3::new(0.0, 0.0, -30.0)]) {
        Ok(points) => points,
        Err(err) => panic!("fixture spawns are valid: {err}"),
    }
}

#[test]
fn a_blocked_spawn_is_refused() {
    let world = World::new(vec![aabb(Vec3::splat(-1.0), Vec3::splat(1.0))]);
    let result = Bot::try_spawn(Vec3::ZERO, &world, RATE, 1);
    assert_eq!(result.err(), Some(SpawnError::Blocked));
}

#[test]
fn a_new_bot_is_whole() {
    let bot = spawn(&world());
    assert_eq!(bot.health().points(), MAX_HEALTH);
    assert_eq!(bot.launcher().ammo(), START_AMMO);
    assert_eq!(bot.score().frags(), 0);
    assert!(bot.as_target().is_some());
}

#[test]
fn a_bot_with_a_target_in_view_shoots_it() {
    let world = world();
    let mut bot = spawn(&world);
    let target = mark(-30.0);
    for _ in 0..600 {
        bot.step(&world, true, Some(&target), &[]);
    }
    assert!(bot.launcher().ammo() < START_AMMO, "the bot never fired");
}

#[test]
fn nothing_moves_while_the_round_is_not_playing() {
    let world = world();
    let mut bot = spawn(&world);
    let before = bot.player().position();
    let target = mark(-30.0);
    for _ in 0..600 {
        bot.step(&world, false, Some(&target), &[]);
    }
    assert_eq!(bot.player().position(), before);
    assert_eq!(bot.launcher().ammo(), START_AMMO);
    assert!(bot.active_body(false).is_none());
}

#[test]
fn a_blast_hurts_a_bot_and_a_killing_one_counts_a_death() {
    let world = world();
    let blast = blast_on_origin(&world);
    let mut bot = spawn(&world);

    let first = bot.take_blast(&blast, &world, false);
    assert_ne!(first, Outcome::Ignored);
    assert!(bot.health().points() < MAX_HEALTH);

    let mut outcome = first;
    for _ in 0..10 {
        if outcome == Outcome::Killed {
            break;
        }
        outcome = bot.take_blast(&blast, &world, false);
    }
    assert_eq!(outcome, Outcome::Killed);
    assert!(!bot.health().is_alive());
    assert_eq!(bot.score().deaths(), 1);
    assert!(bot.active_body(true).is_none());
}

#[test]
fn a_dead_bot_comes_back_far_from_where_it_fell() {
    let world = world();
    let blast = blast_on_origin(&world);
    let spawns = spawns();
    let mut bot = spawn(&world);
    kill(&mut bot, &blast, &world);

    let wait = (RESPAWN_SECONDS * RATE.hz() as f32).ceil() as u32;
    for tick in 1..=wait {
        assert!(bot.step_health(&spawns, &world).is_ok());
        assert_eq!(bot.health().is_alive(), tick == wait);
    }
    assert_eq!(bot.player().position(), Vec3::new(0.0, 0.0, -30.0));
    assert_eq!(bot.launcher().ammo(), START_AMMO);
}

#[test]
fn a_new_round_puts_the_bot_back_home_with_a_clean_score() {
    let world = world();
    let blast = blast_on_origin(&world);
    let mut bot = spawn(&world);
    bot.record_kill();
    kill(&mut bot, &blast, &world);

    assert!(bot.reset(&world).is_ok());

    assert!(bot.health().is_alive());
    assert_eq!(bot.score().kills(), 0);
    assert_eq!(bot.score().deaths(), 0);
    assert_eq!(bot.player().position(), Vec3::ZERO);
}

#[test]
fn pickups_are_wanted_only_when_they_would_change_something() {
    let world = world();
    let mut bot = spawn(&world);
    assert!(!bot.wants(PickupKind::Health));
    assert!(bot.wants(PickupKind::Ammo));

    let blast = blast_on_origin(&world);
    bot.take_blast(&blast, &world, false);
    assert!(bot.wants(PickupKind::Health));
    bot.receive(PickupKind::Health);
    assert!(bot.health().points() > 0);
}
