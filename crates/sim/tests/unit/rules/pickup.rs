use glam::Vec3;
use sim::collision::Aabb;
use sim::combat::Health;
use sim::combat::tuning::MAX_HEALTH;
use sim::rules::tuning::{AMMO_PICKUP_AMOUNT, HEALTH_PICKUP_AMOUNT};
use sim::rules::{Pickup, PickupKind};
use sim::tick::TickRate;
use sim::weapon::Launcher;
use sim::weapon::tuning::{MAX_AMMO, START_AMMO};

const RATE: TickRate = TickRate::DEFAULT;
/// The ticks a health pickup's wait comes to at 120 Hz.
const HEALTH_WAIT_TICKS: u32 = 2400;

fn pickup(kind: PickupKind) -> Pickup {
    match Pickup::try_new(kind, Vec3::ZERO, RATE) {
        Some(pickup) => pickup,
        None => panic!("fixture position is finite"),
    }
}

/// A body standing on the pickup, or well away from it.
fn body(at: Vec3) -> Aabb {
    match Aabb::try_new(at + Vec3::Y, Vec3::new(0.4, 1.0, 0.4)) {
        Ok(body) => body,
        Err(err) => panic!("fixture body is valid: {err}"),
    }
}

#[test]
fn a_non_finite_position_makes_no_pickup() {
    let nan = Vec3::new(f32::NAN, 0.0, 0.0);
    assert!(Pickup::try_new(PickupKind::Ammo, nan, RATE).is_none());
}

#[test]
fn a_touched_pickup_is_taken() {
    let mut pickup = pickup(PickupKind::Health);
    let taken = pickup.step(Some(&body(Vec3::ZERO)), |_| true);
    assert_eq!(taken, Some(PickupKind::Health));
    assert!(!pickup.is_ready());
}

#[test]
fn a_pickup_out_of_reach_stays_ready() {
    let mut pickup = pickup(PickupKind::Health);
    let taken = pickup.step(Some(&body(Vec3::new(5.0, 0.0, 0.0))), |_| true);
    assert_eq!(taken, None);
    assert!(pickup.is_ready());
}

#[test]
fn nobody_takes_nothing() {
    let mut pickup = pickup(PickupKind::Health);
    assert_eq!(pickup.step(None, |_| true), None);
    assert!(pickup.is_ready());
}

#[test]
fn an_unwanted_pickup_is_left_alone() {
    let mut pickup = pickup(PickupKind::Ammo);
    assert_eq!(pickup.step(Some(&body(Vec3::ZERO)), |_| false), None);
    assert!(pickup.is_ready());
}

#[test]
fn a_taken_pickup_returns_after_its_wait() {
    let mut pickup = pickup(PickupKind::Health);
    let on_it = body(Vec3::ZERO);
    pickup.step(Some(&on_it), |_| true);
    for _ in 1..HEALTH_WAIT_TICKS {
        assert_eq!(pickup.step(Some(&on_it), |_| true), None);
    }
    // The tick that ends the wait hands it over again.
    assert_eq!(
        pickup.step(Some(&on_it), |_| true),
        Some(PickupKind::Health)
    );
}

#[test]
fn reset_makes_a_taken_pickup_ready() {
    let mut pickup = pickup(PickupKind::Ammo);
    pickup.step(Some(&body(Vec3::ZERO)), |_| true);
    pickup.reset();
    assert!(pickup.is_ready());
}

#[test]
fn health_is_wanted_only_when_hurt() {
    let launcher = Launcher::new(RATE);
    let mut health = Health::new(RATE);
    assert!(!PickupKind::Health.is_wanted(&health, &launcher));
    health.damage(10);
    assert!(PickupKind::Health.is_wanted(&health, &launcher));
}

#[test]
fn health_is_not_wanted_by_the_dead() {
    let launcher = Launcher::new(RATE);
    let mut health = Health::new(RATE);
    health.damage(MAX_HEALTH);
    assert!(!PickupKind::Health.is_wanted(&health, &launcher));
}

#[test]
fn ammo_is_wanted_only_below_the_cap() {
    let health = Health::new(RATE);
    let mut launcher = Launcher::new(RATE);
    assert!(PickupKind::Ammo.is_wanted(&health, &launcher));
    launcher.add_ammo(MAX_AMMO);
    assert!(!PickupKind::Ammo.is_wanted(&health, &launcher));
}

#[test]
fn giving_restores_health_and_loads_rockets() {
    let mut health = Health::new(RATE);
    let mut launcher = Launcher::new(RATE);
    health.damage(60);
    PickupKind::Health.give(&mut health, &mut launcher);
    PickupKind::Ammo.give(&mut health, &mut launcher);
    assert_eq!(health.points(), MAX_HEALTH - 60 + HEALTH_PICKUP_AMOUNT);
    assert_eq!(launcher.ammo(), START_AMMO + AMMO_PICKUP_AMOUNT);
}
