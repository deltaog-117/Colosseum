//! Hot-path benchmarks for the 120 Hz tick: player movement, collision
//! sweeps, rocket flight and a bot's decision. Run through `scripts/bench`, which gates
//! regressions above 5% against a saved baseline.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use glam::Vec3;
use sim::bot::{Brain, Target};
use sim::collision::{Aabb, Axis3, World};
use sim::input::{Button, Command, Event, InputState};
use sim::movement::{Player, Pose};
use sim::tick::TickRate;
use sim::weapon::Launcher;

const HZ: u32 = 120;
const PILLARS_PER_SIDE: u8 = 6;

fn rate() -> TickRate {
    match TickRate::try_new(HZ) {
        Ok(rate) => rate,
        Err(err) => panic!("benchmark tick rate is valid: {err}"),
    }
}

fn solid(min: Vec3, max: Vec3) -> Aabb {
    match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("benchmark box is valid: {err}"),
    }
}

/// A floor, four walls and a grid of pillars: roughly the solid count of a
/// real arena, so a sweep scans a realistic list.
fn arena() -> World {
    let mut solids = vec![
        solid(Vec3::new(-30.0, -1.0, -30.0), Vec3::new(30.0, 0.0, 30.0)),
        solid(Vec3::new(20.0, 0.0, -30.0), Vec3::new(21.0, 8.0, 30.0)),
        solid(Vec3::new(-21.0, 0.0, -30.0), Vec3::new(-20.0, 8.0, 30.0)),
        solid(Vec3::new(-30.0, 0.0, 20.0), Vec3::new(30.0, 8.0, 21.0)),
        solid(Vec3::new(-30.0, 0.0, -21.0), Vec3::new(30.0, 8.0, -20.0)),
    ];
    for i in 0..PILLARS_PER_SIDE {
        for j in 0..PILLARS_PER_SIDE {
            let (x, z) = (f32::from(i) * 6.0 - 15.0, f32::from(j) * 6.0 - 15.0);
            solids.push(solid(
                Vec3::new(x, 0.0, z),
                Vec3::new(x + 1.5, 4.0, z + 1.5),
            ));
        }
    }
    World::new(solids)
}

fn command(held: &[Button]) -> Command {
    let mut input = InputState::new();
    for &button in held {
        input.apply(Event::Press(button));
    }
    input.take_command()
}

fn spawn(feet: Vec3, world: &World) -> Player {
    match Player::try_spawn(feet, world, rate()) {
        Ok(player) => player,
        Err(err) => panic!("benchmark spawn point is free: {err}"),
    }
}

fn player_step(c: &mut Criterion) {
    let world = arena();
    let mut group = c.benchmark_group("player_step");

    let hop = command(&[Button::Jump]);
    let mut hopper = spawn(Vec3::new(-18.0, 0.0, -18.0), &world);
    group.bench_function("hop", |b| b.iter(|| hopper.step(black_box(&hop), &world)));

    // Pressed into the east wall, so every tick resolves a blocked slide.
    let into_wall = command(&[Button::Right]);
    let mut pusher = spawn(Vec3::new(19.0, 0.0, -18.0), &world);
    group.bench_function("against_wall", |b| {
        b.iter(|| pusher.step(black_box(&into_wall), &world));
    });
    group.finish();
}

fn world_sweep(c: &mut Criterion) {
    let world = arena();
    let probe = solid(Vec3::new(-18.5, 0.0, -18.5), Vec3::new(-17.5, 1.8, -17.5));
    c.bench_function("world_sweep/three_axes", |b| {
        b.iter(|| {
            let x = world.sweep(black_box(&probe), Axis3::X, black_box(5.0));
            let y = world.sweep(black_box(&probe), Axis3::Y, black_box(-5.0));
            let z = world.sweep(black_box(&probe), Axis3::Z, black_box(5.0));
            (x, y, z)
        });
    });
}

fn launcher_step(c: &mut Criterion) {
    let world = arena();
    let aim = match Pose::try_new(Vec3::new(-18.0, 1.6, -18.0), 0.6, 0.1) {
        Some(pose) => pose,
        None => panic!("benchmark pose is finite"),
    };
    let mut launcher = Launcher::new(rate());
    let targets = [solid(Vec3::new(5.0, 0.0, 5.0), Vec3::new(6.0, 1.8, 6.0))];
    // Refilling ammo and firing each tick keeps rockets in flight and
    // detonating, which is the steady state during a fight.
    c.bench_function("launcher_step/in_flight", |b| {
        b.iter(|| {
            launcher.add_ammo(1);
            launcher.step_against(black_box(&world), black_box(&targets));
            let _ = launcher.try_fire(&aim);
        });
    });
}

fn brain_decide(c: &mut Criterion) {
    let world = arena();
    let body = spawn(Vec3::new(-18.0, 0.0, -18.0), &world);
    let mut brain = Brain::new(rate(), 1);
    // Across the pillar grid, so the line-of-sight cast scans a realistic list.
    let target = match Target::try_new(Vec3::new(15.0, 1.6, 15.0), Vec3::new(2.0, 0.0, 0.0)) {
        Some(target) => target,
        None => panic!("benchmark target is finite"),
    };
    c.bench_function("brain_decide/across_the_arena", |b| {
        b.iter(|| brain.decide(black_box(&body), black_box(Some(&target)), &world));
    });
}

criterion_group!(
    benches,
    player_step,
    world_sweep,
    launcher_step,
    brain_decide
);
criterion_main!(benches);
