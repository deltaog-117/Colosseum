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
