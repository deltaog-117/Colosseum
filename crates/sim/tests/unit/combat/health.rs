use sim::combat::tuning::{MAX_HEALTH, RESPAWN_SECONDS};
use sim::combat::{Health, Outcome};
use sim::tick::TickRate;

const RATE: TickRate = TickRate::DEFAULT;
/// The ticks the respawn wait comes to at 120 Hz.
const RESPAWN_TICKS: u32 = 240;

#[test]
fn the_respawn_constant_matches_the_tick_count_used_here() {
    assert_eq!((RESPAWN_SECONDS * RATE.hz() as f32) as u32, RESPAWN_TICKS);
}

#[test]
fn a_new_body_is_alive_at_full_health() {
    let health = Health::new(RATE);
    assert!(health.is_alive());
    assert_eq!(health.points(), MAX_HEALTH);
}

#[test]
fn damage_takes_points_and_the_body_lives() {
    let mut health = Health::new(RATE);
    assert_eq!(health.damage(30), Outcome::Hurt);
    assert_eq!(health.points(), MAX_HEALTH - 30);
    assert!(health.is_alive());
}

#[test]
fn damage_to_the_last_point_kills() {
    let mut health = Health::new(RATE);
    assert_eq!(health.damage(MAX_HEALTH - 1), Outcome::Hurt);
    assert_eq!(health.points(), 1);
    assert_eq!(health.damage(1), Outcome::Killed);
    assert!(!health.is_alive());
    assert_eq!(health.points(), 0);
}

#[test]
fn overkill_does_not_wrap_around() {
    let mut health = Health::new(RATE);
    assert_eq!(health.damage(u16::MAX), Outcome::Killed);
    assert_eq!(health.points(), 0);
}

#[test]
fn zero_damage_changes_nothing() {
    let mut health = Health::new(RATE);
    assert_eq!(health.damage(0), Outcome::Ignored);
    assert_eq!(health, Health::new(RATE));
}

#[test]
fn the_dead_take_no_further_damage() {
    let mut health = Health::new(RATE);
    health.damage(MAX_HEALTH);
    let dead = health;
    assert_eq!(health.damage(10), Outcome::Ignored);
    assert_eq!(health, dead);
}

#[test]
fn a_living_body_does_not_respawn() {
    let mut health = Health::new(RATE);
    health.damage(10);
    assert!(!health.step());
    assert_eq!(health.points(), MAX_HEALTH - 10);
}

#[test]
fn the_dead_respawn_at_full_health_after_the_wait() {
    let mut health = Health::new(RATE);
    health.damage(MAX_HEALTH);
    for tick in 1..RESPAWN_TICKS {
        assert!(!health.step(), "respawned early, at tick {tick}");
        assert!(!health.is_alive());
    }
    assert!(health.step());
    assert!(health.is_alive());
    assert_eq!(health.points(), MAX_HEALTH);
    assert!(!health.step(), "respawn is reported once");
}

#[test]
fn healing_restores_points_up_to_the_maximum() {
    let mut health = Health::new(RATE);
    health.damage(40);
    assert_eq!(health.heal(25), 25);
    assert_eq!(health.points(), MAX_HEALTH - 15);
    assert_eq!(health.heal(100), 15);
    assert_eq!(health.points(), MAX_HEALTH);
}

#[test]
fn healing_the_dead_does_nothing() {
    let mut health = Health::new(RATE);
    health.damage(MAX_HEALTH);
    assert_eq!(health.heal(50), 0);
    assert!(!health.is_alive());
}
