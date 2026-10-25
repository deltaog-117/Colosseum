use glam::Vec3;
use sim::collision::{Aabb, World};
use sim::movement::Pose;
use sim::tick::TickRate;
use sim::weapon::tuning::{DAMAGE, MAX_AMMO, PROJECTILE_SPEED, START_AMMO};
use sim::weapon::{Denied, Launcher};

const RATE: TickRate = TickRate::DEFAULT;
/// The ticks the default cooldown and lifetime come to at 120 Hz.
const COOLDOWN_TICKS: u32 = 96;
const LIFETIME_TICKS: u32 = 600;

fn aabb(min: Vec3, max: Vec3) -> Aabb {
    match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    }
}

/// Open air, no solids at all.
fn open() -> World {
    World::default()
}

/// A wall 20 m ahead of the origin when looking down -Z; its face is at z = -20.
fn wall_ahead() -> World {
    World::new(vec![aabb(
        Vec3::new(-30.0, -1.0, -21.0),
        Vec3::new(30.0, 10.0, -20.0),
    )])
}

fn aim() -> Pose {
    match Pose::try_new(Vec3::new(0.0, 1.6, 0.0), 0.0, 0.0) {
        Some(pose) => pose,
        None => panic!("fixture pose is finite"),
    }
}

fn fire_ok(launcher: &mut Launcher) {
    if let Err(denied) = launcher.try_fire(&aim()) {
        panic!("the launcher should fire: {denied}");
    }
}

fn step_n(launcher: &mut Launcher, world: &World, ticks: u32) {
    for _ in 0..ticks {
        launcher.step(world);
    }
}

#[test]
fn a_new_launcher_is_loaded_and_empty_handed_of_rockets() {
    let launcher = Launcher::new(RATE);
    assert_eq!(launcher.ammo(), START_AMMO);
    assert!(launcher.projectiles().is_empty());
    assert!(launcher.detonations().is_empty());
}

#[test]
fn firing_spends_one_rocket_and_launches_one() {
    let mut launcher = Launcher::new(RATE);
    fire_ok(&mut launcher);
    assert_eq!(launcher.ammo(), START_AMMO - 1);
    assert_eq!(launcher.projectiles().len(), 1);
}

#[test]
fn the_cooldown_denies_a_second_shot_for_exactly_its_length() {
    let mut launcher = Launcher::new(RATE);
    fire_ok(&mut launcher);
    let world = open();

    step_n(&mut launcher, &world, COOLDOWN_TICKS - 1);
    assert_eq!(launcher.try_fire(&aim()), Err(Denied::Cooling));
    assert_eq!(launcher.ammo(), START_AMMO - 1);

    launcher.step(&world);
    assert_eq!(launcher.try_fire(&aim()), Ok(()));
}

#[test]
fn an_empty_launcher_stays_empty() {
    let mut launcher = Launcher::new(RATE);
    let world = open();
    for _ in 0..START_AMMO {
        fire_ok(&mut launcher);
        step_n(&mut launcher, &world, COOLDOWN_TICKS);
    }
    assert_eq!(launcher.ammo(), 0);
    assert_eq!(launcher.try_fire(&aim()), Err(Denied::Empty));
    assert_eq!(launcher.ammo(), 0);
}

#[test]
fn a_rocket_leaves_the_eye_along_the_view_at_full_speed() {
    let mut launcher = Launcher::new(RATE);
    fire_ok(&mut launcher);
    step_n(&mut launcher, &open(), 120);

    let travelled = launcher.projectiles()[0].position() - aim().eye();
    // One second of flight, straight down -Z.
    assert!((travelled - Vec3::new(0.0, 0.0, -PROJECTILE_SPEED)).length() < 1.0e-2);
}

#[test]
fn a_rocket_into_a_wall_detonates_on_its_face_for_full_damage() {
    let mut launcher = Launcher::new(RATE);
    let world = wall_ahead();
    fire_ok(&mut launcher);

    let mut ticks = 0;
    while launcher.detonations().is_empty() && ticks < LIFETIME_TICKS {
        launcher.step(&world);
        ticks += 1;
    }

    let [detonation] = launcher.detonations() else {
        panic!("one rocket should have hit the wall");
    };
    assert!((detonation.position() - Vec3::new(0.0, 1.6, -20.0)).length() < 1.0e-3);
    assert_eq!(detonation.normal(), Vec3::Z);
    assert_eq!(detonation.damage(), DAMAGE);
    assert!(launcher.projectiles().is_empty());
    // 20 m at 25 m/s is 0.8 s, give or take the tick it lands in.
    assert!((95..=98).contains(&ticks), "hit after {ticks} ticks");
}

#[test]
fn detonations_are_reported_for_one_tick_only() {
    let mut launcher = Launcher::new(RATE);
    let world = wall_ahead();
    fire_ok(&mut launcher);
    while launcher.detonations().is_empty() {
        launcher.step(&world);
    }
    launcher.step(&world);
    assert!(launcher.detonations().is_empty());
}

#[test]
fn a_rocket_in_open_air_expires_after_its_lifetime_without_a_bang() {
    let mut launcher = Launcher::new(RATE);
    let world = open();
    fire_ok(&mut launcher);

    step_n(&mut launcher, &world, LIFETIME_TICKS);
    assert_eq!(launcher.projectiles().len(), 1);
    launcher.step(&world);
    assert!(launcher.projectiles().is_empty());
    assert!(launcher.detonations().is_empty());
}

#[test]
fn a_rocket_launched_from_inside_a_solid_detonates_at_once() {
    let block = World::new(vec![aabb(Vec3::splat(-5.0), Vec3::splat(5.0))]);
    let mut launcher = Launcher::new(RATE);
    fire_ok(&mut launcher);
    launcher.step(&block);
    assert_eq!(launcher.detonations().len(), 1);
}

#[test]
fn a_rocket_detonates_on_a_target_in_front_of_the_wall() {
    let world = wall_ahead();
    let target = aabb(Vec3::new(-1.0, 0.0, -11.0), Vec3::new(1.0, 3.0, -10.0));
    let mut launcher = Launcher::new(RATE);
    fire_ok(&mut launcher);
    for _ in 0..LIFETIME_TICKS {
        launcher.step_against(&world, &[target]);
        if let Some(bang) = launcher.detonations().first() {
            assert!((bang.position().z - -10.0).abs() < 1.0e-3);
            assert_eq!(bang.normal(), Vec3::Z);
            assert_eq!(bang.damage(), DAMAGE);
            return;
        }
    }
    panic!("the rocket never hit the target");
}

#[test]
fn a_wall_in_front_of_a_target_stops_the_rocket_first() {
    let world = wall_ahead();
    let hidden = aabb(Vec3::new(-1.0, 0.0, -29.0), Vec3::new(1.0, 3.0, -28.0));
    let mut launcher = Launcher::new(RATE);
    fire_ok(&mut launcher);
    for _ in 0..LIFETIME_TICKS {
        launcher.step_against(&world, &[hidden]);
        if let Some(bang) = launcher.detonations().first() {
            assert!((bang.position().z - -20.0).abs() < 1.0e-3);
            return;
        }
    }
    panic!("the rocket never hit the wall");
}

#[test]
fn added_ammo_fills_up_to_the_cap() {
    let mut launcher = Launcher::new(RATE);
    assert_eq!(launcher.add_ammo(5), 5);
    assert_eq!(launcher.ammo(), START_AMMO + 5);
    assert_eq!(launcher.add_ammo(u16::MAX), MAX_AMMO - START_AMMO - 5);
    assert_eq!(launcher.ammo(), MAX_AMMO);
    assert_eq!(launcher.add_ammo(1), 0);
}
