# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Cargo workspace with three crates: `sim`, `render` and `game`.
- Feature-first folder layout with a `shared/` folder per crate and
  unit, integration and property test trees mirroring the source.
- `scripts/check`: format check, clippy with warnings denied, and the
  workspace test suite.
- Workspace lints forbidding `unsafe` code and denying `unwrap`.
- `ROADMAP.md` and `DIARY.md`.
- `sim::tick`: a validated `TickRate` (1 to 1000 Hz, default 120 Hz) and
  a `FixedStep` accumulator that turns frame times into whole ticks plus
  an interpolation alpha, with long stalls clamped to 250 ms.
- `game`: a `winit` window opened by a thin `main.rs`, and a `GameLoop`
  that drives the accumulator each frame. The window title shows a tick
  counter until there is something to render.
- Unit and property tests for the tick module (alpha range, pending
  time below one step, bounded ticks per frame, time conservation).
- `sim::input`: `Button`, `Event` and an `InputState` that turns events
  into one `Command` per tick (two movement axes, jump, raw look delta).
  Opposing buttons cancel, taps shorter than a tick are kept, look is
  consumed once, and losing focus clears everything.
- `game`: WASD and Space bindings on physical keys, mouse look from raw
  device motion while the cursor is grabbed (click to grab, Escape or
  focus loss to release). The window title shows the current movement
  command as a temporary readout.
- Unit and property tests for the input module.
- `render`: a `wgpu` renderer that draws colored indexed meshes with a
  depth buffer into a 320x240 offscreen target, then scales it onto the
  window with nearest-neighbor filtering (whole-number scale, letterboxed).
- `render::camera`: a validated `Projection` and a first-person `Camera`
  (yaw wraps, pitch is clamped short of vertical).
- `render::mesh`: a validated `Mesh` and a shaded `Mesh::cuboid`.
- `render::resolution`: `RenderResolution` and the window-fitting
  `Viewport` math.
- `game`: the window now renders a placeholder grey-box room, and mouse
  look turns the camera.
- Unit and property tests for the camera, resolution and mesh modules.
- `sim::collision`: a validated `Aabb`, a `World` of solid boxes and a
  per-axis `sweep` that stops a moving box a hair short of any solid, so
  it cannot tunnel at any speed.
- `sim::movement`: a `Player` with Quake-style ground friction, strafe
  acceleration, capped air acceleration (air-strafing), gravity and
  jumping (a tap shorter than a tick still jumps), plus a `Pose` that
  interpolates the view between ticks the short way around the yaw wrap.
  Tuning constants live in `sim::movement::tuning`.
- `render::camera::Camera::set_view`: places the view exactly from
  simulation state.
- `game`: the player now walks the grey-box room with collision (the
  walls, floor, ceiling and pillars are solid), and the view is
  interpolated between ticks.
- Unit and property tests for collision and movement: the player never
  ends up inside a solid, state stays finite and within its limits,
  walking never exceeds top speed, and identical inputs replay to an
  identical player.
- `game::level`: loads a glTF binary (`.glb`) into render meshes, a
  collision `World` and a spawn point. Nodes named `col_*` become box
  colliders (rejected unless their transform keeps them axis-aligned),
  a node named `spawn` places the player, and every other mesh node is
  drawn, shaded by face orientation.
- `assets/levels/arena.glb`: the arena, the same room and pillars as the
  old placeholder, now authored as data.
- Unit and property tests for the loader: colliders contain every placed
  vertex, axis-aligned transforms keep the volume, turns off the axes are
  rejected, and malformed levels fail with an error naming the node.
- `sim::collision`: `World::cast`, an exact segment-versus-box test that
  returns the first hit with its fraction, point and face normal. A very
  fast mover cannot tunnel a thin wall, and the same call will serve
  hitscan weapons and bot line of sight.
- `sim::weapon`: a rocket `Launcher` with ammo (10), a cooldown (0.8 s),
  and projectiles that fly at 25 m/s for up to 5 s, leaving the eye along
  the view. A rocket that meets a solid reports a `Detonation` (position,
  face normal, damage 100) for one tick. A refused shot returns a `Denied`
  reason (empty, cooling, too many in flight). Tuning constants live in
  `sim::weapon::tuning`.
- `sim::input`: a `Fire` button and `Command::fire`, held or tapped like
  jump.
- `sim::movement::Pose::forward`: the unit view direction.
- `render::Renderer::set_transient_meshes`: short-lived meshes, such as
  projectiles, that are replaced on every call.
- `game`: left mouse fires; each rocket is drawn as a small orange cube
  between its last two ticks, and the title shows the ammo left.
- Unit and property tests for the above: hits lie on the segment and not
  inside a solid, a miss never enters a solid, ammo is conserved, shots
  are never closer than the cooldown, rockets never end up inside a solid,
  and rockets in open air fly straight at constant speed.

- `sim::combat`: `Health` (hit points, death, timed respawn, no state
  between alive and dead) and `impact`, which turns a detonation into
  damage and knockback for a body. Strength falls off linearly to zero at
  `BLAST_RADIUS` (3.5 m); walls block it; the shooter takes 50% of the
  damage and all of the knockback via `Impact::to_shooter`.
- `sim::collision::cast_boxes`: segment cast against any set of boxes.
- `sim::weapon::Launcher::step_against`: rockets also detonate on target
  bodies, whichever they meet first.
- `sim::movement::Player::push`: adds a knockback impulse, ignores
  non-finite input and keeps the terminal speed limit.
- `game`: rocket blasts hurt and throw the player (rocket jumping), death
  waits two seconds then respawns at the spawn point with a full
  launcher, and the title shows health.
- Unit and property tests for the above: health stays in bounds and always
  respawns, impacts are finite and within weapon limits, walls block
  blasts, direct hits deal full damage, and pushes respect the limits.
- `sim::rules`: `SpawnPoints` (never empty, always finite; picks the point
  farthest from where a body fell), `Score` (kills, deaths, frags), `Pickup`
  and `PickupKind` (health and ammo, with a respawn wait and a wanted check
  so a full body leaves them), and `Round`, a 180 s clock that ends in an
  8 s intermission and restarts with a clean score.
- `sim::combat::Health::heal` and `sim::weapon::Launcher::add_ammo`, both
  capped (`MAX_HEALTH`, new `MAX_AMMO` of 20).
- `game`: the arena has four spawn points and four pickups; deaths respawn
  far from where the player fell, pickups draw as green and yellow cubes
  while ready, and the title shows time left, kills, deaths and frags.
  When time runs out the player freezes and the final score stays up until
  the next round.
- Unit and property tests for the above: spawn choice is the farthest
  point, pickups cannot be retaken before their wait, phases alternate and
  the score is frozen between rounds, and healing and loading never pass
  their caps.
- `render::hud`: a `HudBatch` of rectangles and text turned into textured
  quads, and a built-in 5x7 bitmap font (digits, letters and a few
  symbols) packed into an `R8` atlas, so the HUD needs no asset files.
- `Renderer::set_hud` and a HUD pass drawn into the low-resolution target
  with alpha blending, before the blit.
- `game::hud`: health with a gauge, rockets, round clock, frags, a
  crosshair, `YOU DIED` and `ROUND OVER` messages, and a scoreboard shown
  while Tab is held and between rounds.
- Unit and property tests for the above: any character maps inside the
  atlas, batches stay whole, finite and within their quad cap, text width
  scales linearly, and every HUD vertex stays on screen for any game state.
- `render::psx`: validated `PsxSettings` and `Fog` for the PS1 look, packed
  into a uniform the scene shader reads. `PsxSettings::disabled()` turns
  every effect off.
- Scene shader: vertex snapping to the 320x240 pixel grid, distance fog and
  4x4 ordered dithering to 5 bits per channel.
- `Renderer::set_psx` to change the look at runtime.
- Unit and property tests for the above: bad fog ranges, colors and dither
  strengths are rejected, and packed settings are always finite.
- `sim` benchmarks (`criterion`): player step, collision sweep and rocket
  flight on a 41-solid arena.
- `scripts/bench`: `save` records a baseline, `compare` fails on a
  regression above 5%. Run by hand when a hot path changes.
- `sim::bot`: a reflex `Brain` that turns a `Target` into the same
  `Command` a keyboard makes (face it at a limited turn rate, lead a moving
  target, hold a fighting distance, strafe, hop when boxed in, fire only
  with a clear line and no closer than the blast reach), and a `Bot` that
  owns a body, launcher, health and score.
- `Command::from_intent`: builds a command from code, with the look
  cleaned like a device look.
- `sim::combat::strike`: one blast against one body, shared by the player
  and the bots.
- `game`: three bots in the arena. They fight the nearest living body,
  take pickups they touch, respawn, reset each round and are drawn as red
  boxes. Rockets now burst on bodies and kills are credited to the shooter.
- Scoreboard rows for each bot.
- `brain_decide` benchmark.
- Unit and property tests for the above.

### Changed
- Rockets fired by the player now burst on bots as well as on walls.
- `HudView` carries the bots' score rows.
- Mouse look now turns the player in `sim` rather than the camera in
  `game`; the look sensitivity constant moved with it.
- `sim` now depends on `glam` for vector math.
- The level contract accepts several `spawn` / `spawn_*` nodes and
  `pickup_health*` / `pickup_ammo*` nodes; `arena.glb` gained three spawns
  and four pickups. A level with two spawn nodes is no longer an error.
- `game` now depends on `gltf` (without its image and base64 features),
  and loads the level from the embedded arena instead of `grey_box`.
- The scene is drawn with snapping, fog and dithering by default, and the
  background clears to the fog color while fog is on.

### Removed
- `game::grey_box`: replaced by `game::level` and `arena.glb`.
- The window title readout (ticks, input, ammo, health, round and score),
  replaced by the HUD; the title is now just `fps-3d-game`.
