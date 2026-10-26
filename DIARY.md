# 📘 DIARY.md

## Architectural Decision Log

*A chronological record of decisions, trade-offs and reasoning. Git tracks
what changed; this diary tracks why.*

Author: deltaog-117

---

## 📋 Decision Index

| Date | Decision Area | Choice | Status |
|------|---------------|--------|--------|
| 2026-10-25 | Genre, language, engine, art | Single-player arena shooter, Rust, custom `wgpu` engine, PS1 look | ✅ Confirmed |
| 2026-10-25 | Workspace layout | Three crates, feature-first folders, tiered tests | ✅ Confirmed |
| 2026-10-25 | Project check command | `scripts/check`, Tier 1 only | ✅ Confirmed |
| 2026-10-25 | Fixed-timestep loop | Pure accumulator in `sim/tick`, `winit` in `game`, 120 Hz | ✅ Confirmed |
| 2026-10-25 | Input sampling | Device-neutral `Event` and `InputState` in `sim/input`, one `Command` per tick | ✅ Confirmed |
| 2026-10-25 | Minimal renderer | Direct `wgpu`, scene into a low-res target, nearest-neighbor blit | ✅ Confirmed |
| 2026-10-25 | Player movement and collision | Quake-style movement, per-axis swept AABB collision in `sim` | ✅ Confirmed |
| 2026-10-25 | Hit detection and damage | Splash damage and knockback on box bodies, `sim::combat` | ✅ Confirmed |
| 2026-10-26 | Performance benchmarks | `criterion` in `sim`, manual `scripts/bench` gate at 5% | ✅ Confirmed |
| 2026-10-26 | Bots | Generated navigation grid, built in slices; slice 1 is the reflex bot | 🟡 Slice 1 done |

---

## 📝 Decision Entries

### Core Design Decisions

**Date:** 2026-10-25
**Status:** Confirmed

#### Context / Background

Before scaffolding, the shape of the game had to be fixed so the roadmap
could be ordered around it.

#### Decision & Rationale

- **Genre:** single-player Quake-style arena shooter; networking comes
  last, once the game is fun.
- **Language:** Rust, edition 2024.
- **Dimension:** straight to 3D, no 2D prototype.
- **Engine:** custom, on `wgpu` and `winit`, with small helpers (`gltf`,
  `image`, `glam`, `rodio`). Accepted trade-off: more work than an
  existing engine, in exchange for full control of the simulation/render
  split and the PS1 look.
- **First weapon:** projectiles; hitscan can come later.
- **Art:** low-poly Quake/PS1 flavor, grey-box placeholders until the
  game is fun.

#### Review / Update Log

| Date | Update | Author |
|------|--------|--------|
| 2026-10-25 | Initial entry | deltaog-117 |

---

### Workspace Layout: Three Crates, Feature-First

**Date:** 2026-10-25
**Status:** Confirmed

#### Context / Background

The architecture rules require feature-first folders, a read-only
`shared/`, and tests that mirror the source. The roadmap also requires a
simulation with no graphics dependency, so art can be swapped later and
the same code can run on a server.

#### Options Considered

**Option A: One crate, feature folders inside it**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Simplest manifest <br> • Fastest to start |
| **Disadvantages** | • Nothing stops the sim from importing `wgpu` <br> • Server reuse needs a later split |
| **Implementation Difficulty** | Easy |

**Option B: Three crates (`sim`, `render`, `game`)**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • The compiler enforces "sim has no graphics dependency" <br> • `sim` is reusable by a future server <br> • `render` can be tested apart from game rules |
| **Disadvantages** | • Cross-crate APIs must be designed up front <br> • Slightly more manifest boilerplate |
| **Implementation Difficulty** | Medium |

#### Decision & Rationale

**Chosen Option:** B.

The no-graphics rule is the project's central constraint, and a crate
boundary is the only way to enforce it mechanically. `game` depends on
`sim` and `render`; those two never depend on each other.

**Trade-offs accepted:** boundary cost now, in exchange for a sim that
stays headless and testable.

#### Implementation Notes

- `sim/src`: `arena`, `collision`, `input`, `movement`, `tick`, `shared`.
- `render/src`: `camera`, `mesh`, `low_res_target`, `shared`.
- `game/src`: `window`, `device_input`, `game_loop`, `shared`.
- Tests mirror the source under `tests/unit`, `tests/integration` and
  (sim only) `tests/property`; `sim/benches` holds Tier 2 benchmarks.
- `movement` depends on `collision`, one way only. Deleting `movement`
  leaves a compiling sim; deleting `collision` does not. This is the one
  accepted exception to the delete test.
- A first attempt created flat `tests/<feature>/` folders and a
  `lowres_target` spelling that duplicated an earlier scaffold. The
  duplicates were empty and were removed; the earlier layout was kept.
- `sim/src/world` was dropped for now; level geometry arrives at v0.2.
- Workspace lints forbid `unsafe_code` and deny `unwrap_used`.

#### Review / Update Log

| Date | Update | Author |
|------|--------|--------|
| 2026-10-25 | Initial entry | deltaog-117 |

---

### Project Check Command

**Date:** 2026-10-25
**Status:** Confirmed

#### Context / Background

The workflow needs exactly one check command that a hook can also run.

#### Decision & Rationale

**Chosen Option:** `scripts/check`, a thin shell wrapper running
`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets
-- -D warnings` and `cargo test --workspace -q`.

This is Tier 1 only. Clippy already covers typechecking. A secret scan
and a dependency audit are not configured yet and are left out rather
than invented. Benchmarks (Tier 2) are added once a hot path is
benchmarked.

#### Implementation Notes

The script passes with no tests present; it currently proves only that
format, lint and build are clean.

#### Review / Update Log

| Date | Update | Author |
|------|--------|--------|
| 2026-10-25 | Initial entry | deltaog-117 |

---

### Feature: Fixed-Timestep Loop

**Date:** 2026-10-25
**Status:** Confirmed

#### Context / Background

State must advance in fixed ticks independent of the frame rate, and the
renderer needs an interpolation factor between ticks. The `winit` event
loop drives the window.

#### Options Considered

**Option A: Pure accumulator in `sim/tick`; `winit` handler in `game`**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Property-testable with no window <br> • Keeps the sim free of graphics code <br> • Reusable by a future server |
| **Disadvantages** | • The API between two crates is designed up front |
| **Implementation Difficulty** | Medium |

**Option B: Accumulator and `winit` handler both in `game`**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Fewest files <br> • Fastest to a window |
| **Disadvantages** | • Timing logic testable only with a window <br> • Server would duplicate it |
| **Implementation Difficulty** | Easy |

**Option C: Sim on its own thread, snapshots over a channel**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Rendering never stalls the sim |
| **Disadvantages** | • Needs a formal concurrency model <br> • Snapshot plumbing before any gameplay exists |
| **Implementation Difficulty** | Hard |

#### Decision & Rationale

**Chosen Option:** A, at **120 Hz**. 120 divides evenly into 60, 120 and
240 Hz displays, so ticks and frames line up on common monitors.

**Trade-offs accepted:** the sub-nanosecond rounding in the step (below) and
single-threaded sim and render, which is revisited only if profiling
shows the frame budget is at risk.

#### Implementation Notes

- `TickRate` is a validated newtype (1 to 1000 Hz); `try_new` is the only
  way to build one besides the `DEFAULT` constant.
- The tick step is integer nanoseconds (8,333,333 ns at 120 Hz). That
  rounded value is the definition of a tick, so no drift accumulates;
  the sim runs about 0.04 ppm fast against the wall clock, which is
  irrelevant for a single-player game.
- Elapsed time per frame is clamped to 250 ms (about 30 ticks at 120 Hz).
  A stall drops time rather than running a catch-up burst that would
  make the next frame slower still (the "spiral of death").
- `alpha` is documented as `0.0..=1.0`, inclusive, because converting the
  `f64` ratio to `f32` can round a value just below 1 up to 1.
- `GameLoop::frame` takes the `Instant` as a parameter so tests drive the
  clock. `game` is a binary crate, so its tests live in the module.
- Property tests: alpha range, pending time below one step, ticks per
  frame bounded, and time conservation (`ticks * step + pending` equals
  the total elapsed) for unclamped input.
- Not yet done, on purpose: logging and tracing (nothing worth tracing
  until there is state), and a benchmark for `advance` (it is a handful of
  integer operations; benchmarks arrive with the v0.2 to v0.4 work).
- `main` returns `Box<dyn Error>` rather than adding `anyhow` for one
  call site; revisit when error sources multiply.
- The post-write formatter hook orders imports in the 2021 style, while
  `cargo fmt` uses 2024. `cargo fmt --all` before the check settles it.

#### Review / Update Log

| Date | Update | Author |
|------|--------|--------|
| 2026-10-25 | Initial entry | deltaog-117 |

---

### Feature: Input Sampling

**Date:** 2026-10-25
**Status:** Confirmed

#### Context / Background

The simulation must never read devices; it consumes one small command per
tick. Frames and ticks do not line up (a frame can run zero, one or
several ticks), so the rules for turning key and mouse events into
per-tick commands need to be exact and testable.

#### Options Considered

**Option A: `Command` in `sim`, sampling rules in a `game`-side sampler**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Smallest change <br> • Little new `sim` surface |
| **Disadvantages** | • Latching and delta rules sit outside the property-test tree <br> • Easy to get the frame-to-tick split subtly wrong |
| **Implementation Difficulty** | Easy |

**Option B: `Button`, `Event`, `InputState` and `Command` all in `sim/input`**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • All input rules property-testable headless <br> • `game` only translates winit events <br> • Rebinding touches `game` only <br> • Reusable by a future server |
| **Disadvantages** | • More `sim` surface up front |
| **Implementation Difficulty** | Medium |

**Option C: Timestamped raw events applied at exact tick boundaries**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Sub-frame accuracy <br> • Natural base for netcode replay |
| **Disadvantages** | • Queue and timestamp plumbing is heavy at 120 Hz <br> • Benefit only arrives with networking |
| **Implementation Difficulty** | Hard |

#### Decision & Rationale

**Chosen Option:** B. It is the only option that puts the "sim never
reads devices" boundary in the type system, and it puts the fiddly rules
under the existing property tests.

**Trade-offs accepted:** more `sim` surface now; input is sampled per
frame rather than per event, which is fine at 120 Hz and revisited with
networking (option C).

#### Implementation Notes

- `Command` has private fields and is built only by
  `InputState::take_command`. Movement is two `Axis` values (negative,
  zero, positive), so "forward and back at once" cannot be represented:
  opposing buttons resolve to `Zero`.
- `take_command` drains the one-shot parts (mouse movement and presses)
  and keeps held buttons. When a frame runs several ticks, only the
  first carries the look delta, so a long frame does not multiply a turn.
- A press is latched until the next command, so a tap shorter than a
  tick is seen by exactly one command instead of being lost.
- Look is raw device counts; sensitivity and pitch clamping belong to the
  movement step. Non-finite motion is dropped and the sum is capped at
  `LOOK_LIMIT`, so a command's look is always finite.
- `FocusLost` clears held buttons, latches and pending look, so a key
  released while another window had focus cannot stay stuck.
- `game` maps physical keys (WASD, Space) so AZERTY keeps the layout.
  Mouse motion comes from `DeviceEvent::MouseMotion` and is applied only
  while the cursor is grabbed (click to grab, Escape or focus loss to
  release). Key repeats are skipped.
- Cursor grab tries `Locked` and falls back to `Confined` (X11).
- Until a renderer or movement exists, the window title shows the last
  command's forward, strafe and jump, as a manual check.
- Property tests: look finite and bounded, look consumed once, a tap
  seen by exactly one command, one button activates only itself, held
  buttons survive many commands, focus loss returns to idle.
- Not yet done: the grab and title wiring in `window.rs` has no
  automated test (it needs a window); the pure mapping is unit-tested.

#### Review / Update Log

| Date | Update | Author |
|------|--------|--------|
| 2026-10-25 | Initial entry | deltaog-117 |

---

### Feature: Minimal Renderer

**Date:** 2026-10-25
**Status:** Confirmed

#### Context / Background

`render` was an empty stub. v0.1 needs colored static meshes with depth, a
first-person camera and the low-resolution, nearest-neighbor look, and the
PS1 shader pass (v0.2) must have a scene pass of its own to extend.

#### Options Considered

**Option A: Direct `wgpu`, two passes (scene to a low-res target, then a blit)**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Matches the locked custom-engine decision <br> • PS1 effects slot into the scene pass <br> • Few dependencies |
| **Disadvantages** | • Two pipelines and two shaders <br> • GPU code is hard to unit test |
| **Implementation Difficulty** | Medium |

**Option B: Single pass at native resolution**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Smallest first step |
| **Disadvantages** | • Misses the low-res requirement <br> • Reworks the pipeline and depth buffer later |
| **Implementation Difficulty** | Easy |

**Option C: A higher-level engine crate**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Fastest to pixels |
| **Disadvantages** | • Contradicts the custom-engine decision <br> • Hides the pipeline PS1 effects need <br> • Large dependency tree |
| **Implementation Difficulty** | Easy |

#### Decision & Rationale

**Chosen: Option A.** Both the low-res target and the later PS1 effects
depend on the scene being its own pass; doing it now avoids reworking
pipelines and the depth buffer. B only defers the same work and C breaks
a decision already made.

**Trade-offs accepted:** more boilerplate now, and the `wgpu` part is
verified by running the game, not by automated tests.

#### Implementation Notes

- Pure, GPU-free types carry the rules and are property-tested:
  `Projection` (fov in `(0, π)`, `0 < near < far`), `Camera` (pitch
  clamped short of vertical, yaw wrapped, non-finite turns ignored),
  `RenderResolution` and its `fit` (largest whole scale that fits,
  centered, scales down only for a window smaller than the target) and
  `Mesh` (non-empty, whole triangles, indices in range, finite values).
- `Renderer` takes anything `wgpu` accepts as a surface target, so `game`
  hands it an `Arc<Window>` and `render` never depends on `winit`.
- The offscreen color target shares the surface format, so the blit is a
  plain copy; the sampler is nearest. The projection uses the target's
  aspect, not the window's, and the viewport letterboxes the rest.
- Camera convention: right-handed, +Y up, yaw 0 looks down -Z, positive
  yaw turns left, positive pitch looks up. `game` flips mouse signs.
- Depth range is 0..1 and Y-up (`glam`'s `directx` projection), matching
  `wgpu`. `glam` 0.34 moved these under `glam::camera`.
- No back-face culling yet: the grey-box room is seen from the inside and
  placeholder winding is not worth enforcing.
- `game::grey_box` is a placeholder room with pillars; real geometry and
  collision replace it later. Mouse look turns the camera directly, per
  tick, until the player exists in `sim`.
- Not done: interpolation between ticks (nothing moves yet), a benchmark
  (no measured hot path yet) and any automated test of the GPU code.
  `scripts/check` does not open a window, so the picture was checked by
  running the game only.

#### Review / Update Log

| Date | Update | Author |
|------|--------|--------|
| 2026-10-25 | Initial entry | deltaog-117 |

---

### Feature: Player Movement and Collision

**Date:** 2026-10-25
**Status:** Confirmed

#### Context / Background

The last open v0.1 item. Movement feel is the heart of an arena shooter,
and the simulation has to stay free of graphics so art can change without
touching game logic and the same code can later run on a server.

#### Options Considered

**Option A: Quake movement with per-axis swept box collision**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Small and easy to property-test <br> • Wall sliding comes for free <br> • Fits the box-only room |
| **Disadvantages** | • Box solids only, so the collision half is replaced for glTF levels |
| **Implementation Difficulty** | Easy |

**Option B: Same movement with time-of-impact sweeps and slide-plane clipping**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Handles ramps and general planes <br> • Carries over to level geometry |
| **Disadvantages** | • Much more math and corner cases <br> • More invariants to prove |
| **Implementation Difficulty** | Hard |

**Option C: `rapier3d` kinematic character controller**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Mature collision and slopes |
| **Disadvantages** | • Heavy dependency in `sim` <br> • Not Quake feel <br> • Harder to keep deterministic |
| **Implementation Difficulty** | Medium |

#### Decision & Rationale

**Chosen: Option A.** It gives the Quake feel at the lowest risk, and the
v0.1 room is only boxes. Collision sits behind `World::sweep`, so level
geometry can replace it without touching the movement code.

**Trade-offs accepted:** no ramps or stairs until level geometry exists.

#### Implementation Notes

- `sim::collision`: `Aabb` is stored as center plus half extents, so
  moving it never changes its size by rounding. `World::sweep` moves one
  axis at a time and measures the gap to every solid beside the path, so
  a step of any length stops at the nearest face instead of tunneling.
  The box stops `SKIN` (0.1 mm) short; without it rounding could leave it
  overlapping by a hair, and an overlapping box is stuck. A solid that
  already overlaps or lies behind never blocks, so a box can back out.
- `sim::movement`: `Player` follows Quake's order: turn, jump, then
  friction and accelerate on the ground (jumping first skips friction, so
  chained jumps keep their speed) or air-accelerate with a small speed cap
  plus gravity in the air. The cap is what makes air-strafing curve a jump
  without free steering. Wish direction is normalized, so diagonals are no
  faster. Each axis is then swept in X, Z, Y order and a blocked axis loses
  its velocity, which makes walls slide.
- Ground detection probes 2 cm down and also snaps the player onto the
  surface, so it never hovers inside the probe distance.
- Per-axis speed is clamped to `TERMINAL_SPEED`, which keeps long falls
  and strafe-jump chains finite. Ground speed is only bounded by
  `MAX_SPEED` on the ground: running into a wall at an angle can slide
  faster, as in Quake.
- Units are meters and seconds; the constants are tuning values in
  `sim::movement::tuning`, not settings.
- Mouse look moved into `Player`, since movement needs the yaw. `PITCH_LIMIT`
  and the wrap are duplicated from `render::camera`; a `game` test asserts
  the limits match.
- `Pose` is the eye position plus yaw and pitch. `lerp` takes the short
  arc across ±π, and `game` renders `lerp(previous, current, alpha)`, one
  tick behind the simulation, which is the usual price of smooth motion.
- `game::grey_box` builds both the meshes and the collision world from
  the same constants, so what is drawn is what is solid.
- Not done: a benchmark (the roadmap schedules benchmarks with the tick
  loop and collision regression gate), logging (no logging infrastructure
  exists yet), stairs and ramps, and a check of the feel by running the
  game; `scripts/check` does not open a window.

#### Review / Update Log

| Date | Update | Author |
|------|--------|--------|
| 2026-10-25 | Initial entry | deltaog-117 |

---

### Feature: Level Geometry and Collision

**Date:** 2026-10-25
**Status:** Confirmed

#### Context / Background

First v0.2 item. The v0.1 room was hard-coded constants; levels need to
be data, with one file driving both what is drawn and what is solid.

#### Options Considered

**Option A: glTF renders the level; `col_*` box proxies collide**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • `World` and the axis sweep stay as they are <br> • Existing movement property tests still apply <br> • Smallest change |
| **Disadvantages** | • Axis-aligned geometry only, no ramps <br> • Two sets of geometry to keep in step |
| **Implementation Difficulty** | Easy |

**Option B: Triangle-soup collision with a grid or BVH**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Any mesh is walkable <br> • No proxies |
| **Disadvantages** | • Movement needs a slide-plane rewrite <br> • Most edge cases and tests |
| **Implementation Difficulty** | Hard |

**Option C: Quake-style convex brushes**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Robust ramps and angled walls |
| **Disadvantages** | • New plane-clip sweep and movement rewrite <br> • Hull authoring in glTF |
| **Implementation Difficulty** | Medium |

#### Decision & Rationale

**Chosen: Option A.** It ships a real glTF level without touching the
movement code that already passes its property tests, and the loader,
render path and asset pipeline are needed under any option. Option C can
extend it later: `col_*` nodes could carry brushes behind the same
`World` interface.

**Trade-offs accepted:** no ramps or angled walls until brush or
triangle collision exists.

#### Implementation Notes

- `game::level` stays inside `game`, the glue between `sim` and `render`;
  neither of those learns about glTF. The level is one `.glb` embedded
  with `include_bytes!`, so there is no working-directory dependence and
  no external buffer or image files (the `gltf` crate is built without
  its `import` feature for the same reason).
- Contract: `col_*` mesh nodes are colliders and are not drawn, a `spawn`
  node is the player start, any other mesh node is drawn. A level with no
  spawn, two spawns or no colliders is rejected.
- A collider is the bounds of its placed vertices. Bounds only equal the
  shape when the transform keeps it on the axes, so any turn other than a
  quarter turn is an error rather than a silently bigger box.
- Drawn meshes take the material's base color factor and are shaded by
  the face normal with the same table as `Mesh::cuboid`, since there are
  no lights yet. Textures, vertex colors and non-triangle modes are not
  read.
- `arena.glb` was generated from the old placeholder constants by a
  throwaway script, so the room, pillars and spawn match the old one and
  the tests assert the same counts (6 meshes, 10 solids). The script is
  not in the repo; edit levels in a glTF tool from here on.
- `game::grey_box` is deleted.
- Not done: a benchmark (the roadmap schedules benchmarks with the
  collision regression gate), logging (no logging infrastructure yet),
  and a run of the game to look at the arena; `scripts/check` does not
  open a window.

#### Review / Update Log

| Date | Update | Author |
|------|--------|--------|
| 2026-10-25 | Initial entry | deltaog-117 |

### Feature: Projectile Weapon

**Date:** 2026-10-25
**Status:** Confirmed

#### Context / Background

Second v0.2 item and the core of the game. Hit detection, bots, rules
and the HUD ammo count all build on a weapon that spawns, flies and
detonates deterministically inside `sim`.

#### Options Considered

**Option A: Small box moved with the existing `World::sweep`**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Reuses tested collision <br> • Smallest diff |
| **Disadvantages** | • Per-axis sweeps can clip corners <br> • Speed and size need a cap <br> • No reuse for hitscan or line of sight |
| **Implementation Difficulty** | Easy |

**Option B: Segment cast (slab test), the projectile is a point**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Exact first hit, point and normal at any speed <br> • Reusable for hitscan and bot line of sight |
| **Disadvantages** | • One new module and its property tests |
| **Implementation Difficulty** | Medium |

**Option C: Generic entity store (e.g. `hecs`)**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Later entities slot in without new containers |
| **Disadvantages** | • New dependency <br> • Abstraction before a second entity type exists <br> • Larger diff in `game` |
| **Implementation Difficulty** | Hard |

#### Decision & Rationale

**Chosen: Option B.** It is the only option that gives the exact first-hit
point at rocket speeds, which is where splash damage will start, and the
cast is reused by hitscan and bot line of sight. Option A would likely be
replaced when hit detection arrives.

**Trade-offs accepted:** one more collision primitive to maintain.

#### Implementation Notes

- `World::cast` lives in `sim::collision::ray`. A segment that starts on
  a face and leaves, or only grazes an edge, hits nothing; one that starts
  inside a solid hits at fraction 0 with the normal pointing back along
  the segment, so a rocket fired from inside geometry explodes at once.
- `Launcher` owns the ammo, cooldown and a bounded pool of `Projectile`s.
  The projectile list and the detonation list are reused every tick, so
  stepping does not allocate. Call order per tick is `step`, then
  `try_fire`.
- Rockets leave the eye, not a muzzle offset, so a shot against a wall
  cannot skip past it. Rockets do not hit the player yet.
- Speeds are in meters: 25 m/s is about 2.8x the player's top speed,
  taken from Quake's ratio of rocket to run speed rather than its raw 900
  units. The cooldown rounds up to whole ticks (96 at 120 Hz).
- A `Denied` reason is returned for refused shots. `game` ignores it for
  now; the HUD and audio will use it.
- Left mouse fires, held or tapped like jump. The click that grabs the
  cursor does not fire. Rockets are drawn as cubes through
  `Renderer::set_transient_meshes`, which uploads buffers every frame;
  instancing will replace it when the shader pass lands.
- Not done: a benchmark (scheduled with the collision regression gate),
  logging (no logging infrastructure yet), applying damage (next item),
  and a run of the game to look at the rockets; `scripts/check` does not
  open a window.

#### Review / Update Log

| Date | Update | Author |
|------|--------|--------|
| 2026-10-25 | Initial entry | deltaog-117 |

---

### Feature: Hit Detection and Damage

**Date:** 2026-10-25
**Status:** Confirmed

#### Context / Background

Third v0.2 item. Rockets detonated on walls and carried a damage number
that nothing applied. Players are `Aabb` boxes, not capsules, and there is
only one body in the game so far.

#### Options Considered

**Option A: Direct hit only, segment versus box, health and respawn**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Smallest change <br> • Reuses `Aabb` and the slab test |
| **Disadvantages** | • A near miss does nothing <br> • No rocket jumping |
| **Implementation Difficulty** | Simple |

**Option B: Splash damage with falloff and knockback on box bodies**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Gives the rocket its feel <br> • Direct hit falls out as distance zero <br> • Uses only existing types |
| **Disadvantages** | • Radius, falloff and knockback to tune <br> • Needs a line-of-sight test <br> • Touches `Player` velocity |
| **Implementation Difficulty** | Medium |

**Option C: Capsule shapes, spatial grid, then health and respawn**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Matches the original roadmap <br> • Scales to many bots |
| **Disadvantages** | • Largest scope, speculative with one body <br> • Two shapes, as movement stays a box |
| **Implementation Difficulty** | Hard |

#### Decision & Rationale

**Chosen: Option B, with knockback included.** The rocket launcher is a
splash weapon and does not feel right without it. Capsules and the grid
wait for bots, when there are enough bodies to justify them.

**Trade-offs accepted:** the roadmap's capsule and spatial grid are
deferred; bodies stay boxes.

#### Implementation Notes

- `combat::impact(detonation, world, body)` measures from the blast to the
  nearest point of the box. Strength is `1 - reach / BLAST_RADIUS`, and
  both damage and knockback scale with it. Knockback points from the blast
  through the body's center, so a rocket at the feet throws the player up.
- Line of sight is a world cast from 5 cm in front of the struck face to
  the body's center. The lift matters: from the face itself the cast
  starts on the wall and would slip through a thin one.
- The shooter takes 50% of the damage and all of the knockback, so a rocket
  jump hurts but cannot kill from full health. Point-blank at the feet is
  50 damage; two of them in a row kill.
- `Health` is `Alive(1..=100)` or `Dead(ticks)`, so a body cannot be alive
  with no points. The dead ignore damage. Respawn waits 2 s.
- `Launcher::step_against(world, targets)` casts against the target boxes
  as well as the world and takes the nearer hit. `step` is the same with no
  targets. The shooter's own box is left out, since a rocket starts
  inside it; its own blast still reaches it through `impact`.
- In `game`, death freezes input and the player; rockets already flying
  continue. Respawn builds a new `Player` and a fresh `Launcher`.
- Tuning values (radius 3.5 m, knockback 12 m/s, 2 s respawn) are first
  guesses, not playtested.
- Not done: no other body exists yet, so direct hits are only covered by
  tests; no benchmark (scheduled with the collision gate); no logging; and
  the game was not run to feel the rocket jump, since `scripts/check` does
  not open a window.

#### Review / Update Log

| Date | Update | Author |
|------|--------|--------|
| 2026-10-25 | Initial entry | deltaog-117 |

---

### Feature: Game Rules

**Date:** 2026-10-25
**Status:** Confirmed

#### Context / Background

Fourth v0.2 item. The game had one spawn point, a respawn timer inside
`Health`, ammo that only went down and no score, pickups or end to the
match. There is still one body, so nothing can score a kill yet.

#### Options Considered

**Option A: A headless `sim::rules` module with a round state machine**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Matches the sim-has-no-graphics rule <br> • Property-testable <br> • Reusable by bots, the HUD and a server |
| **Disadvantages** | • Largest API surface <br> • `game` needs glue to drive it |
| **Implementation Difficulty** | Medium |

**Option B: Thin slices in existing modules**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Smallest diff <br> • Each piece lands alone |
| **Disadvantages** | • Match state has no single owner <br> • Bots and the HUD would force a refactor |
| **Implementation Difficulty** | Simple |

**Option C: Event-driven rules layer**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Clean seam for HUD, audio and netcode |
| **Disadvantages** | • More abstraction than one player justifies |
| **Implementation Difficulty** | Hard |

#### Decision & Rationale

**Chosen: Option A.** The HUD and bots both read match state, so it needs
one owner. An event stream can grow out of it later if audio or netcode
want one.

**Trade-offs accepted:** `Round` scores one player; per-player scores wait
for bots.

#### Implementation Notes

- `Round` is `Playing(ticks)` or `Intermission(ticks)`, both always
  positive, so a round cannot sit at zero. Phases repeat, and the score
  resets on the tick the next round starts. Kills and deaths are ignored
  outside `Playing`, so the final score cannot move.
- `Pickup` is ready or counting down. `PickupKind::is_wanted` keeps a full
  body from wasting one; `give` applies it through `Health::heal` and
  `Launcher::add_ammo`, both capped.
- `SpawnPoints` is non-empty and finite by construction, so `first()` and
  `farthest_from()` cannot fail. Respawn picks the point farthest from where
  the player fell; a tie goes to the earliest point.
- The level contract takes `spawn` and `spawn_*`, `pickup_health*` and
  `pickup_ammo*` nodes. The old duplicate-spawn error is gone. I added
  seven empty nodes to `arena.glb` by editing its JSON chunk; no buffer
  changed.
- In `game`, a finished round freezes the player and blasts, and the next
  round resets health, launcher, position and every pickup. The title shows
  time, kills, deaths and frags until the HUD exists.
- Tuning values (180 s round, 8 s intermission, +25 health, +5 rockets, 20 s
  and 15 s pickup waits, cap of 20 rockets) are first guesses, not
  playtested.
- Not done: no kill can be earned until bots exist (`Round::record_kill` is
  tested but unused by `game`); no benchmark; no logging; and the game was
  not run to see the pickups, since `scripts/check` does not open a window.

#### Review / Update Log

| Date | Update | Author |
|------|--------|--------|
| 2026-10-25 | Initial entry | deltaog-117 |

---

### Feature: HUD

**Date:** 2026-10-25
**Status:** Confirmed

#### Context / Background

Fifth v0.2 item. Health, ammo, time and score lived in the window title,
and the player had no crosshair. `sim::rules` already exposed all of the
numbers; the renderer had two passes (scene, blit) and no textures or text.

#### Options Considered

**Option A: HUD pass into the 320x240 target, with a quad batcher and a built-in 8x8 bitmap font atlas**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Matches the PS1 look <br> • No new dependencies or assets <br> • Reuses the existing target and letterbox |
| **Disadvantages** | • Chunky text at 320x240 <br> • The glyph table is hand-written |
| **Implementation Difficulty** | Medium |

**Option B: Same batcher and font, drawn after the blit at window resolution**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Crisp text at any window size |
| **Disadvantages** | • Two coordinate systems <br> • Breaks the low-res look <br> • The letterbox needs extra handling |
| **Implementation Difficulty** | Medium |

**Option C: `egui` or `glyphon`**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Fastest route to a scoreboard and nice text |
| **Disadvantages** | • Heavy dependency tied to the `wgpu` 30 version <br> • Off-style <br> • Against the small-helper-crates rule |
| **Implementation Difficulty** | Medium |

#### Decision & Rationale

**Chosen: Option A.** The roadmap already described the HUD as a font
atlas plus quad batching, and drawing into the low-res target gives the
look for free. The font is pure data, so the layout can be property-tested.

**Trade-offs accepted:** text is blocky and only covers digits, letters and
`: - / . + ! %`; anything else draws as `?`, and lowercase draws as upper.

#### Implementation Notes

- `render::hud::HudBatch` holds whole quads (four vertices each), is capped
  at `MAX_QUADS`, and ignores non-finite or degenerate input instead of
  storing it, so the GPU buffers (sized for the cap) cannot overflow. Text
  snaps to whole pixels so scaled glyphs keep even texels.
- `render::hud::font` packs 45 glyphs plus a solid cell into a 64x48 `R8`
  atlas built at startup. Rectangles sample the middle of the solid cell,
  so one pipeline and one draw call serve both rectangles and text.
- `Renderer::set_hud` uploads the batch; `render` runs a third pass
  (`LoadOp::Load` on the scene target, alpha blending) before the blit. The
  index buffer is built once; only vertices are rewritten per frame.
- `game::hud` maps a `HudView` (plain numbers copied from the match) to
  quads: health with a gauge, rockets, clock, frags, crosshair, `YOU DIED`,
  `ROUND OVER`, and a scoreboard. The scoreboard shows while Tab is held and
  always between rounds. Tab is interface state kept in `App`, so `sim`
  input is untouched. Counts on the scoreboard are clamped to four digits.
- The title is a plain `fps-3d-game` again, and the tick counter and input
  readout are gone with it.
- Checked: property tests for the font and batch (any `char` maps inside
  the atlas, batches stay whole and finite, widths scale linearly) and a
  property test that every HUD vertex stays on the 320x240 screen for any
  game state. I started the game for ten seconds and saw no shader or
  pipeline error, but I have not looked at the picture, so the layout and
  colors are unreviewed by eye.
- Not done: no benchmark; the scoreboard has one row until bots exist; no
  damage flash, pickup messages or kill feed.

#### Review / Update Log

| Date | Update | Author |
|------|--------|--------|
| 2026-10-25 | Initial entry | deltaog-117 |

---

### Feature: PS1 Shader Pass

**Date:** 2026-10-25
**Status:** Confirmed

#### Context / Background

Sixth v0.2 item. The scene shader drew flat vertex colors. The roadmap asked
for vertex snapping, affine texturing, dithering and fog, so the look is in
place before art arrives. Vertices carry a position and a color only, so
affine texturing has nothing to warp yet.

#### Options Considered

**Option A: Constants baked into the WGSL**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Fewest changes <br> • No new types |
| **Disadvantages** | • Every tweak needs a rebuild <br> • Nothing testable from Rust |
| **Implementation Difficulty** | Small |

**Option B: Validated settings type, uploaded as a uniform**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Bad values cannot be built <br> • Tunable and property-testable <br> • Ready for menus and debug tools |
| **Disadvantages** | • A new type, buffer and bind group entry |
| **Implementation Difficulty** | Medium |

**Option C: Settings plus UVs and affine texturing**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • The complete PS1 look |
| **Disadvantages** | • Changes the public `Vertex`, the loader and assets <br> • Mixes in texture work the roadmap leaves for later |
| **Implementation Difficulty** | High |

#### Decision

**Chosen: Option B.** It covers the effects that can be seen today, makes
invalid fog and dither values unrepresentable, and leaves `Vertex`, the
loader and the HUD alone.

**Trade-offs accepted:** affine texturing is not done; it moves to its own
roadmap item for when textures arrive.

#### Implementation Notes

- `render::psx::PsxSettings::try_new` takes the snap grid (cells per target
  pixel, or none), an optional `Fog` and a dither strength in `0..=1`. `Fog`
  needs `0 <= start < end` and a color within `0..=1`. NaN fails every check.
- `PsxSettings::pack` builds three `vec4`s for the shader, so there is no
  std140 padding to get wrong. The snap grid is packed per resolution.
- The vertex stage snaps screen position to the grid and undoes the divide;
  vertices at or behind the camera plane are left alone, as they have no
  screen position. Fog uses view depth (`clip.w`) and is applied before
  dithering. Dithering is a 4x4 Bayer threshold at 5 bits per channel.
- `Renderer::new` starts with `PsxSettings::default()` (whole-pixel snap, fog
  from 12 to 45 units, full dither). `Renderer::set_psx` changes it, and
  `PsxSettings::disabled()` is the rollback. While fog is on, the clear color
  follows the fog color so distant walls meet the background without a seam.
- Checked: unit and property tests for validation and packing; `scripts/check`
  passes; the shader validates under `naga` 30.0.1 (same version as `wgpu`).
  I started the game for ten seconds but the run printed nothing at all, not
  even the GPU line, so it proves nothing about the pipeline. I have not seen
  the picture, so the fog distances and the look are unreviewed by eye.
- Not done: no benchmark; no settings menu; no affine texturing.

#### Review / Update Log

| Date | Update | Author |
|------|--------|--------|
| 2026-10-25 | Initial entry | deltaog-117 |

---

### Feature: Performance Benchmarks

**Date:** 2026-10-26
**Status:** Confirmed

#### Context / Background

Seventh v0.2 item, taken before bots so there is a baseline to compare their
load against. The engineering rules ask for a benchmark on each hot path and
a failure above a 5% regression. `crates/sim/benches/` was empty.

#### Options Considered

**Option A: `criterion` with a saved baseline, run by hand**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Real timings with statistics <br> • Baseline comparison and a noise threshold built in |
| **Disadvantages** | • Manual, because timings are noisy <br> • Baseline is machine-specific |
| **Implementation Difficulty** | Small |

**Option B: `divan` plus a hand-written gate script**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Lighter benchmark code |
| **Disadvantages** | • No baseline comparison, so the gate is custom code <br> • Not in the local cargo cache |
| **Implementation Difficulty** | Medium |

**Option C: Operation-count tests inside `scripts/check`**

| Aspect | Assessment |
|--------|------------|
| **Advantages** | • Runs on every edit, no noise |
| **Disadvantages** | • Counts work, not time <br> • Needs counters inside the collision code |
| **Implementation Difficulty** | Medium |

#### Decision

**Chosen: Option A.** It matches the Tier 2 rule (benchmarks run when a hot
path changes) and needs no custom comparison code.

**Trade-offs accepted:** the gate is not automatic; a slowdown is caught only
when someone runs `scripts/bench compare`.

#### Implementation Notes

- `crates/sim/benches/tick.rs` holds four benchmarks on a 41-solid arena:
  `player_step/hop`, `player_step/against_wall`, `world_sweep/three_axes` and
  `launcher_step/in_flight`.
- `scripts/bench save` records the `main` baseline; `scripts/bench compare`
  re-runs against it with a 5% noise threshold and exits 1 if criterion
  reports "Performance has regressed".
- First numbers on this machine (120 Hz tick budget is 8.3 ms): hop 71 ns,
  against wall 96 ns, three-axis sweep 258 ns, launcher step 89 ns. The tick
  is nowhere near its budget, so bots have plenty of room.
- `criterion` is a dev-dependency of `sim` only, without its plotting and
  rayon features. Nothing changes in the shipped game.
- Checked: save then compare back-to-back reported no regression (changes
  within about 2%); `scripts/check` passes. A real regression was not
  induced, so the failing path of the script is untested.
- Not done: no render benchmark; no CI wiring; the baseline is not committed.

#### Review / Update Log

| Date | Update | Author |
|------|--------|--------|
| 2026-10-26 | Initial entry | deltaog-117 |

---

### Feature: Bots (slice 1 of 2)

**Date:** 2026-10-26
**Status:** Slice 1 confirmed; slice 2 pending

#### Context / Background

The last open v0.2 item. Rockets (`Launcher::step_against`) and kills
(`Round::record_kill`) were already written for other bodies, but
`window.rs` held exactly one player, launcher and health, and a `Command`
could only be built from devices.

#### Options Considered

**Option A: Reflex bot, no navigation.** Simplest; gets stuck in concave
shapes and ignores pickups.

**Option B: Hand-placed waypoints in the glTF.** Designed routes; needs
authoring per level and a loader change.

**Option C: Generated navigation grid.** No authoring and works on any
level; the most code, 2D only.

#### Decision

**Chosen: Option C, in two slices.** Slice 1 is the plumbing every option
needs plus the reflex bot (A). Slice 2 adds the grid behind the same
`Brain` interface.

**Trade-offs accepted:** until slice 2 bots can get stuck behind walls, and
they pick up only what they run into.

#### Implementation Notes

- A bot is a `Player` driven by a `Command`, so it follows the exact
  movement and collision rules. `Command::from_intent` is the only new way
  in, and it applies the same look clamp as a device.
- The brain inverts the player's look rule to turn at a finite rate
  (`TURN_RATE`, 6 rad/s). An instant turn would make every shot land.
- Aim leads the target by the rocket's flight time. Fire needs a clear
  `World::cast`, a view within `FIRE_CONE` and a distance of at least
  `MIN_FIRE_RANGE`, which sits above the blast reach so a bot does not
  rocket itself.
- Strafe timing uses a seeded xorshift, so a bot's choices replay exactly
  and tests stay deterministic. `rand` was not added.
- The jump probe fires only when a wall stops most of the next step along
  both horizontal axes, because a bot pressed against a flat wall slides
  along it. In practice that means corners and dead ends.
- Blast handling moved into `combat::strike`, used by the player and the
  bots alike. Rockets now burst on bodies: the player's against bots, each
  bot's against everyone but itself. Free-for-all; a kill is credited to the
  shooter unless they killed themselves.
- Pickups go to the player first, then the bots in order. A pickup still
  waiting is not offered on, since each `Pickup::step` also runs its clock.
- `brain_decide`: about 0.9 us per decision on a 41-solid arena, against an
  8.3 ms tick. The existing hot paths did not change, so the 5% gate was
  not re-run; `scripts/bench save` will pick up the new benchmark.
- Found while testing: a fixture put the wall 20 m from the bot, so a
  blast was out of reach and an unbounded kill loop hung the suite. The
  helper is now bounded.
- Not done: no navigation, no pickup seeking, no skill levels, no bot
  names beyond "BOT n", and the bot count is a constant.

#### Review / Update Log

| Date | Update | Author |
|------|--------|--------|
| 2026-10-26 | Initial entry, slice 1 | deltaog-117 |
