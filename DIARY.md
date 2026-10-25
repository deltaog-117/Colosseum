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
