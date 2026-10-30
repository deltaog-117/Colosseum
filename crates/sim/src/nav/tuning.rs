//! Navigation constants in meters. Tuning values, not settings yet.

/// Side of one grid cell. Must not exceed the player's width, so a body
/// that fits at two neighbouring centres fits everywhere between them.
pub const CELL_SIZE: f32 = 0.5;
/// How far above the floor the probe box starts, so touching the floor is
/// not read as being inside it.
pub const PROBE_LIFT: f32 = 0.05;
/// How far below the probe box a floor must lie for a cell to count as
/// something to stand on.
pub const PROBE_DROP: f32 = 0.1;
/// How many rings of cells around a blocked cell are searched for a
/// walkable one, when a start or goal point lands in a wall.
pub const SNAP_RINGS: u32 = 4;
/// Most cells a grid may hold, which bounds its memory and build time.
pub const MAX_CELLS: u64 = 1 << 20;
