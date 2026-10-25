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

### Changed
- Mouse look now turns the player in `sim` rather than the camera in
  `game`; the look sensitivity constant moved with it.
- `sim` now depends on `glam` for vector math.
