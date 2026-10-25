use glam::Vec3;
use sim::collision::World;
use sim::movement::Pose;
use sim::tick::TickRate;
use sim::weapon::Launcher;
use sim::weapon::tuning::PROJECTILE_SPEED;

const EYE: Vec3 = Vec3::new(1.0, 2.0, 3.0);

/// A rocket that has flown exactly one tick from `EYE`, looking down -Z.
fn one_tick_old() -> sim::weapon::Projectile {
    let mut launcher = Launcher::new(TickRate::DEFAULT);
    let aim = match Pose::try_new(EYE, 0.0, 0.0) {
        Some(pose) => pose,
        None => panic!("fixture pose is finite"),
    };
    if let Err(denied) = launcher.try_fire(&aim) {
        panic!("the launcher should fire: {denied}");
    }
    launcher.step(&World::default());
    launcher.projectiles()[0]
}

#[test]
fn a_fresh_rocket_has_not_moved() {
    let mut launcher = Launcher::new(TickRate::DEFAULT);
    let aim = match Pose::try_new(EYE, 0.0, 0.0) {
        Some(pose) => pose,
        None => panic!("fixture pose is finite"),
    };
    assert_eq!(launcher.try_fire(&aim), Ok(()));
    let rocket = launcher.projectiles()[0];
    assert_eq!(rocket.position(), EYE);
    assert_eq!(rocket.previous(), EYE);
}

#[test]
fn one_tick_moves_it_one_step_along_the_view() {
    let rocket = one_tick_old();
    let step = PROJECTILE_SPEED / 120.0;
    assert_eq!(rocket.previous(), EYE);
    assert!((rocket.position() - (EYE + Vec3::NEG_Z * step)).length() < 1.0e-5);
}

#[test]
fn position_at_blends_between_the_last_two_ticks() {
    let rocket = one_tick_old();
    let middle = rocket.previous().lerp(rocket.position(), 0.5);
    assert!((rocket.position_at(0.5) - middle).length() < 1.0e-6);
    assert_eq!(rocket.position_at(0.0), rocket.previous());
    assert_eq!(rocket.position_at(1.0), rocket.position());
}

#[test]
fn position_at_clamps_alpha_and_ignores_a_non_finite_one() {
    let rocket = one_tick_old();
    assert_eq!(rocket.position_at(-3.0), rocket.previous());
    assert_eq!(rocket.position_at(3.0), rocket.position());
    assert_eq!(rocket.position_at(f32::NAN), rocket.previous());
}
