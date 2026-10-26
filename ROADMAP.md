# Roadmap

This document outlines the future direction of fps-3d-game, a 3D
Quake-style arena shooter in Rust with a custom engine.
Items are organized by priority, not by timeline.

Author: deltaog-117

---

## ✅ Completed (Milestones Achieved)

- ✅ Design decisions locked (rationale goes into `DIARY.md`):
  - Genre: single-player Quake-style arena shooter, networking last.
  - Language: Rust, edition 2024.
  - Dimension: straight to 3D, no 2D prototype.
  - Engine: custom, on `wgpu` + `winit`, with small helper crates
    (`gltf`, `image`, `glam`, `rodio`).
  - First weapon: projectiles. Hitscan can come later.
  - Art: low-poly Quake / PS1 flavor (low-res textures, vertex
    snapping, affine texturing, low render resolution). Grey-box
    placeholders until the game is fun.
- ✅ Playbook templates digested
- ✅ Project scaffold: Cargo workspace with `sim`, `render` and `game`,
  feature-first folders and mirrored test trees
- ✅ `scripts/check` approved and in place (format, clippy, tests)
- ✅ Window and fixed-timestep loop: 120 Hz `FixedStep` accumulator in
  `sim`, `winit` window and `GameLoop` in `game`
- ✅ Input sampling: `sim::input` turns device-neutral events into one
  `Command` per tick; `game` translates WASD, Space and mouse motion
- ✅ Minimal renderer: `wgpu` scene pass into a 320x240 target, depth,
  first-person camera, nearest-neighbor blit; `game` shows a placeholder
  grey-box room with mouse look
- ✅ Player movement and collision: Quake-style acceleration, friction,
  air control and jumping in `sim`, swept box collision against the
  grey-box room, and tick interpolation of the view
- ✅ Level geometry and collision: `game::level` loads a glTF binary into
  render meshes, box colliders (`col_*` nodes) and a spawn point; the
  shipped arena replaces the hard-coded room
- ✅ Hit detection and damage: rockets detonate on bodies as well as walls;
  the blast deals distance-falloff damage and knockback, walls block it,
  and `Health` carries death and a timed respawn (`sim::combat`)
- ✅ Game rules: several spawn points with far-from-death respawns, health
  and ammo pickups, kills/deaths/frags and a repeating 180 s round
  (`sim::rules`)
- ✅ HUD: health, rockets, round clock, frags, crosshair and a Tab
  scoreboard drawn from a built-in bitmap font into the low-res target
  (`render::hud`, `game::hud`)
- ✅ PS1 shader pass: vertex snapping, distance fog and ordered dithering in
  WGSL, tuned by validated `PsxSettings` (`render::psx`); affine texturing
  waits for texture coordinates
- ✅ Performance benchmarks: `criterion` benches for player step, sweep and
  rocket flight in `sim`; `scripts/bench` gates regressions above 5%

---

## 🔥 High Priority (Critical)

**v0.1: a window with a player that moves in a grey-box room.**

- ✅ **Window and fixed-timestep loop** – State advances in fixed ticks
  (60-128/s) independent of frame rate; rendering interpolates between
  ticks. Everything else builds on this.
- ✅ **Input sampling** – Keys and mouse become a small command struct per
  tick, so the simulation never reads devices directly.
- ✅ **Minimal renderer** – Draw colored static meshes with depth, a
  first-person camera, and a low-resolution offscreen target scaled with
  nearest-neighbor filtering.
- ✅ **Grey-box room and player movement** – Quake-style acceleration,
  friction and jumping, with box collision against the room. Movement
  feel is the heart of an arena shooter.
- ✅ **Simulation crate with no graphics dependency** – The sim knows
  positions, hitboxes and states only, so art is swapped in later
  without touching game logic and the same code can run on a server.
- ✅ **Project check command** – `scripts/check` wrapping format check,
  clippy and tests, plus property tests for movement invariants.

---

## 🟡 Medium Priority (Important)

**v0.2 to v0.4: the rest of a complete single-player game.**

- ✅ **Level geometry and collision** – Real level meshes loaded from glTF
  with box-proxy collision (`col_*` nodes), replacing the single box
  room. Ramps and angled walls wait for brush or triangle collision.
- ✅ **Projectile weapon** – Projectiles with speed, lifetime, cooldown,
  ammo and damage; a rocket-style first weapon.
- ✅ **Hit detection and damage** – Splash damage and knockback against
  box bodies, health, death and respawn. Capsule shapes and a spatial grid
  are deferred until bots make them worth it.
- ✅ **PS1 shader pass** – Vertex snapping, dithering and fog in WGSL.
  Affine texturing is deferred until vertices carry texture coordinates.
- **Bots** – Navigation, aiming and basic behavior, so the game is
  testable alone. Skeletal animation waits until here, as it is the
  largest single renderer piece.
  - ✅ Slice 1: reflex bots that aim, lead, strafe, shoot, take pickups
    and score (`sim::bot`).
  - Slice 2: generated navigation grid and A* pathing, so bots can reach
    targets around walls and run for pickups.
- ✅ **Game rules** – Spawns, respawns, scoring, pickups and a match timer.
- ✅ **HUD** – Health, ammo, crosshair and scoreboard via a font atlas and
  quad batching.
- ✅ **Performance benchmarks** – Benchmarks for the tick loop and
  collision with a 5% regression gate.

---

## 🟢 Low Priority (Nice‑to‑Have)

- **Audio** – Shots, hits, footsteps and ambience through `rodio`.
- **Particles and effects** – Explosions, muzzle flashes, trails.
- **Menus** – Main menu, settings, key rebinding.
- **More weapons and pickups** – Additional projectile weapons, a
  hitscan weapon, armor and powerups.
- **Skeletal animation** – Run, aim and fire clips for bots, skinned and
  blended in the renderer.
- **Local multiplayer or split input** – Optional, after bots exist.
- **Debug tools and hot reload** – Asset reloading and in-game
  inspection overlays.
- **Asset license tracking** – A record of each asset's license (CC0 vs
  attribution required).

---

## 🔭 Long‑Term Vision

**Networking, built last and only once the single-player game is fun.**

- **Dumb client-server first** – Server-authoritative; clients send
  inputs, not positions; the server simulates and broadcasts snapshots
  at a fixed rate.
- **Client prediction and reconciliation** – For the local player.
- **Interpolation** – For remote players.
- **Lag compensation** – The server rewinds time for fair hit detection
  if hitscan weapons exist.
- **Formal modeling of the netcode** – A short state model of the
  protocol before implementation.
- **Level editor or BSP-style level pipeline** – Custom tooling for
  making maps.

---

## 🎯 Next Actions (Immediate)

Bots are half done. Pick the next one:

1. **Bots, slice 2: navigation grid** – generate a walkable grid from the
   box world (`World::sweep`), search it with A*, and let `Brain` follow
   the path instead of walking straight. Fixes bots stuck behind walls.
2. **Affine texturing** – add texture coordinates to `Vertex` and the
   glTF loader once real textures arrive.
