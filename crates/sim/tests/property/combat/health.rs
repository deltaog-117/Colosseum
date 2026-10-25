use proptest::prelude::*;
use sim::combat::Health;
use sim::combat::tuning::MAX_HEALTH;
use sim::tick::TickRate;

/// Damage to deal this tick, if any.
fn plays() -> impl Strategy<Value = Vec<Option<u16>>> {
    prop::collection::vec(prop::option::of(0u16..=300), 1..2000)
}

proptest! {
    #[test]
    fn points_stay_within_bounds_and_match_the_living(play in plays()) {
        let mut health = Health::new(TickRate::DEFAULT);
        for damage in play {
            if let Some(amount) = damage {
                health.damage(amount);
            }
            health.step();
            prop_assert!(health.points() <= MAX_HEALTH);
            prop_assert_eq!(health.is_alive(), health.points() > 0);
        }
    }

    #[test]
    fn damage_never_raises_health(first in 0u16..=300, second in 0u16..=300) {
        let mut health = Health::new(TickRate::DEFAULT);
        health.damage(first);
        let before = health.points();
        health.damage(second);
        prop_assert!(health.points() <= before);
    }

    #[test]
    fn a_killed_body_always_comes_back_at_full_health(overkill in 100u16..=u16::MAX) {
        let mut health = Health::new(TickRate::DEFAULT);
        health.damage(overkill);
        prop_assert!(!health.is_alive());
        let mut ticks = 0u32;
        while !health.step() {
            ticks += 1;
            prop_assert!(ticks < 10_000, "never respawned");
        }
        prop_assert_eq!(health.points(), MAX_HEALTH);
    }
}

proptest! {
    #[test]
    fn healing_never_passes_the_maximum_and_never_lowers_health(
        damage in 0u16..=300,
        amount in 0u16..=u16::MAX,
    ) {
        let mut health = Health::new(TickRate::DEFAULT);
        health.damage(damage);
        let before = health.points();
        let healed = health.heal(amount);
        prop_assert!(health.points() <= MAX_HEALTH);
        prop_assert_eq!(health.points(), before + healed);
    }
}
