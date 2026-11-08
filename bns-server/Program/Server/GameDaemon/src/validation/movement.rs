//! Partial PC movement checks. Success here is not full movement approval.
//! TODO ❓ 381 0x14048C0B0 transition matrix, delayed checks, summoned rules,
//! complete Walk2 wire fields and geometry loading are not mapped (part 1).

use crate::Position;

mod height;
mod speed;
pub use height::*;
pub use speed::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveType {
    pub code: i8,
    pub start_cell: i8,
    pub start_time: i8,
    pub flag: bool,
}

/// Unknown reason numbers are not replaced with invented client reasons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Reason(i16),
    UnmappedReason(UnmappedGate),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnmappedGate {
    PositionCell,
    Layer,
    Environment,
    HeightRange,
    StartDistance,
    Speed,
    ZeroDurationDisplacement,
    VerticalIntervals,
}

/// Separate parser bounds, including the incremented-count check.
/// Spec: Step 5 part 1, Input limits; 381 0x1403892E0.
/// This consumes decoded counts; it does not decode the unknown Walk2 layout.
pub fn handler_counts(
    move_types: usize,
    declared_cells: usize,
    parsed_cells: usize,
) -> Result<(), Failure> {
    if move_types > 12 {
        return Err(Failure::Reason(103));
    }
    if declared_cells > 36 || parsed_cells >= 36 || parsed_cells != declared_cells {
        return Err(Failure::Reason(102));
    }
    Ok(())
}

/// Action bounds differ from the handler bounds. Zero duration is permitted.
/// `None` cells represents the native null cell pointer.
/// Spec: Step 5 part 1, Input limits; 381 0x14048A1E0, handler 0x1403892E0.
/// TODO ❓ Duration-ring setup, action creation and queue integration.
pub fn initializer_limits(
    supplied_zone: Option<u64>,
    creature_zone: Option<u64>,
    native_kind: u8,
    cells: Option<&[Cell]>,
    duration: u32,
    move_types: usize,
) -> Result<(), Failure> {
    if supplied_zone.is_none()
        || supplied_zone != creature_zone
        || !(1..=3).contains(&native_kind)
        || !cells.is_some_and(|c| (1..=36).contains(&c.len()))
        || duration > 550
        || !(1..=12).contains(&move_types)
    {
        return Err(Failure::Reason(2));
    }
    Ok(())
}

/// Selector 1 requires summoned control state 1 or 2. Others select the PC.
/// Spec: Step 5 part 1, Entry points; 381 0x1403892E0.
pub fn selects_summon(selector: u8, summon_control_state: Option<u8>) -> Result<bool, Failure> {
    if selector != 1 {
        return Ok(false);
    }
    if matches!(summon_control_state, Some(1 | 2)) {
        Ok(true)
    } else {
        Err(Failure::Reason(2))
    }
}

/// Spec: Step 5 part 1, Entry points; 381 0x1403892E0.
pub fn selected_creature(native_kind: u8, action_byte_56: u8) -> Result<(), Failure> {
    if native_kind == 2 && action_byte_56 == 11 {
        Err(Failure::Reason(166))
    } else {
        Ok(())
    }
}

/// Cardinal steps, nonnegative x/y, allowed type codes and ordered markers.
/// Call after initializer limits. An empty cell/type list is still rejected.
/// Spec: Step 5 part 1, Cell pseudo-steps 1–6; 381 0x14048A890,
/// 0x14048F620, 0x14048F810.
pub fn records(cells: &[Cell], types: &[MoveType], duration: u32) -> Result<(), Failure> {
    if cells.is_empty() || types.is_empty() {
        return Err(Failure::Reason(2));
    }
    for pair in cells.windows(2) {
        let dx = (i32::from(pair[1].x) - i32::from(pair[0].x)).abs();
        let dy = (i32::from(pair[1].y) - i32::from(pair[0].y)).abs();
        if !matches!((dx, dy), (1, 0) | (0, 1)) {
            return Err(Failure::Reason(117));
        }
    }
    if cells.iter().any(|c| c.x < 0 || c.y < 0) {
        return Err(Failure::Reason(118));
    }
    for (i, t) in types.iter().enumerate() {
        if !valid_move_type(t.code) {
            return Err(Failure::Reason(119));
        }
        if t.start_cell < 0 || t.start_cell as usize > cells.len() {
            return Err(Failure::Reason(120));
        }
        if t.start_time < 0 || 5 * t.start_time as u32 > duration {
            return Err(Failure::Reason(121));
        }
        if i > 0 && t.start_cell < types[i - 1].start_cell {
            return Err(Failure::Reason(122));
        }
        if i > 0 && t.start_time < types[i - 1].start_time {
            return Err(Failure::Reason(123));
        }
    }
    Ok(())
}

/// Spec: Step 5 part 1, Cell pseudo-step 3; 381 0x14048A890.
pub fn valid_move_type(code: i8) -> bool {
    matches!(code, 0..=18 | 22..=36 | 40..=42)
}

/// Opaque geometry fields. Attribute +11 is not replaced with invented flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layer {
    pub floor: i16,
    pub ceiling: i16,
    pub environment: u64,
    pub attribute_11: u8,
}

/// Spec: Step 5 part 1, Cell source; 381 0x14048DE30, 0x1400DD7D0,
/// 0x1400D9E80. TODO ❓ Native geometry/attribute file loading is not mapped.
pub trait Geometry {
    fn origin(&self) -> [i32; 2];
    /// The vertical query interval is half-open. Bounds are widened, not wrapped.
    fn layer(&self, x: i16, y: i16, min_z: i32, max_z: i32) -> Option<Layer>;
    fn environment_enabled(&self, environment: u64) -> bool;
}

/// Endpoint cell mapping, fallback layer and inclusive height bounds.
/// Returns adjusted cell and position without changing request input.
/// Spec: Step 5 part 1, Cell pseudo-steps 8–12; 381 0x14048DE30.
pub fn endpoint<G: Geometry>(
    geometry: &G,
    mut cell: Cell,
    mut position: Position,
) -> Result<(Cell, Position, Layer), Failure> {
    let origin = geometry.origin();
    let x = (i64::from(position.x) - i64::from(origin[0])) / 8;
    let y = (i64::from(position.y) - i64::from(origin[1])) / 8;
    if x != i64::from(cell.x) || y != i64::from(cell.y) {
        return Err(Failure::UnmappedReason(UnmappedGate::PositionCell));
    }
    let allowed =
        |layer: &Layer| layer.environment == 0 || geometry.environment_enabled(layer.environment);
    let exact = geometry
        .layer(cell.x, cell.y, i32::from(cell.z), i32::from(cell.z) + 1)
        .filter(allowed);
    let layer = if let Some(layer) = exact {
        layer
    } else {
        let layer = geometry
            .layer(
                cell.x,
                cell.y,
                i32::from(cell.z) - 10,
                i32::from(cell.z) + 10,
            )
            .ok_or(Failure::UnmappedReason(UnmappedGate::Layer))?;
        if !allowed(&layer) {
            return Err(Failure::UnmappedReason(UnmappedGate::Environment));
        }
        if position.z == cell.z {
            position.z = layer.floor;
        }
        cell.z = layer.floor;
        let adjusted = geometry
            .layer(cell.x, cell.y, i32::from(cell.z), i32::from(cell.z) + 1)
            .ok_or(Failure::UnmappedReason(UnmappedGate::Layer))?;
        if !allowed(&adjusted) {
            return Err(Failure::UnmappedReason(UnmappedGate::Environment));
        }
        adjusted
    };
    if position.z < layer.floor || position.z > layer.ceiling {
        return Err(Failure::UnmappedReason(UnmappedGate::HeightRange));
    }
    Ok((cell, position, layer))
}

/// Resolve later path cells against server geometry and environment state.
/// Spec: Step 5 part 1, Cell pseudo-step 16; 381 0x14048A890.
pub fn path_layer<G: Geometry>(geometry: &G, cell: Cell) -> Result<Layer, Failure> {
    let layer = geometry
        .layer(cell.x, cell.y, i32::from(cell.z), i32::from(cell.z) + 1)
        .ok_or(Failure::UnmappedReason(UnmappedGate::Layer))?;
    if layer.environment != 0 && !geometry.environment_enabled(layer.environment) {
        return Err(Failure::UnmappedReason(UnmappedGate::Environment));
    }
    Ok(layer)
}

/// Preserve the recovered conjunction exactly; do not infer new cell flags.
/// Spec: Step 5 part 1, Cell pseudo-step 16; 381 0x14048A890.
pub fn vertical_intervals(origin: Layer, next: Layer) -> Result<(), Failure> {
    if origin.floor >= next.ceiling && origin.ceiling <= next.floor {
        Err(Failure::UnmappedReason(UnmappedGate::VerticalIntervals))
    } else {
        Ok(())
    }
}

/// Start tolerance uses signed-short creature +2304 plus 5, and z tolerance 10.
/// Spec: Step 5 part 1, Cell pseudo-step 13; 381 0x14048A890.
pub fn start_position(request: Position, server: Position, short_2304: i16) -> Result<(), Failure> {
    let limit = i32::from(short_2304) + 5;
    let dx = (i32::from(request.x) - i32::from(server.x)).abs();
    let dy = (i32::from(request.y) - i32::from(server.y)).abs();
    let dz = (i32::from(request.z) - i32::from(server.z)).abs();
    if dx > limit || dy > limit || dz > 10 {
        Err(Failure::UnmappedReason(UnmappedGate::StartDistance))
    } else {
        Ok(())
    }
}

/// Spec: Step 5 part 1, Cell pseudo-step 15; 381 0x14048A890.
pub fn zero_duration(duration: u32, start: Position, end: Position) -> Result<(), Failure> {
    if duration == 0 && start != end {
        Err(Failure::UnmappedReason(
            UnmappedGate::ZeroDurationDisplacement,
        ))
    } else {
        Ok(())
    }
}

/// Spec: Step 5 part 1, Cell pseudo-step 7; 381 0x14048A890.
pub fn transition_counter(request: u16, server: u16) -> Result<(), Failure> {
    if request == server {
        Ok(())
    } else {
        Err(Failure::Reason(104))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Correction {
    pub command: i16,
    pub reason: i16,
    pub movement: u8,
    pub position: Position,
    pub transition_counter: u16,
    pub end_action: bool,
}

/// Server correction plan, never a kick. Counter mismatch must not increment.
/// Unmapped reason numbers cannot produce a fabricated wire reply.
/// Spec: Step 5 part 1, Failure result; 381 0x14048DC90, 0x140422850,
/// counter mismatch 0x14048A890. Format hEchhhh.
pub fn correction(
    failure: Failure,
    movement: u8,
    server: Position,
    counter: u16,
) -> Option<Correction> {
    let Failure::Reason(reason) = failure else {
        return None;
    };
    Some(Correction {
        command: 65,
        reason,
        movement,
        position: server,
        transition_counter: if reason == 104 {
            counter
        } else {
            counter.wrapping_add(1)
        },
        end_action: true,
    })
}

/// Handler parse/create/initialize/queue failures have zero correction fields.
/// Spec: Step 5 part 1, Input limits; 381 0x14038BE00, 0x1403892E0.
pub fn handler_failure(reason: i16) -> Correction {
    Correction {
        command: 65,
        reason,
        movement: 0,
        position: Position::default(),
        transition_counter: 0,
        end_action: false,
    }
}
