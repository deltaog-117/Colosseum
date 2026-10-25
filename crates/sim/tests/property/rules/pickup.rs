use glam::Vec3;
use proptest::prelude::*;
use sim::collision::Aabb;
use sim::rules::{Pickup, PickupKind};
use sim::tick::TickRate;

fn kinds() -> impl Strategy<Value = PickupKind> {
    prop_oneof![Just(PickupKind::Health), Just(PickupKind::Ammo)]
}

/// Per tick: where the body stands, and whether it wants the pickup.
fn plays() -> impl Strategy<Value = Vec<(f32, bool)>> {
    prop::collection::vec((-1.0f32..1.0, any::<bool>()), 1..4000)
}

proptest! {
    #[test]
    fn a_taken_pickup_is_never_taken_again_before_its_wait_ends(
        kind in kinds(),
        play in plays(),
    ) {
        let rate = TickRate::DEFAULT;
        let mut pickup = Pickup::try_new(kind, Vec3::ZERO, rate)
            .ok_or_else(|| TestCaseError::fail("origin is finite"))?;
        // The shortest wait is 15 s; takes closer than that are a bug.
        let shortest_wait = 15 * rate.hz() as usize;
        let mut last_take: Option<usize> = None;
        for (tick, (x, wants)) in play.into_iter().enumerate() {
            let body = Aabb::try_new(Vec3::new(x, 1.0, 0.0), Vec3::new(0.4, 1.0, 0.4))
                .map_err(|err| TestCaseError::fail(err.to_string()))?;
            if pickup.step(Some(&body), |_| wants).is_some() {
                if let Some(previous) = last_take {
                    prop_assert!(tick - previous >= shortest_wait);
                }
                last_take = Some(tick);
            }
        }
    }
}
