use super::{valid_move_type, Failure, MoveType, UnmappedGate};
use crate::Position;

/// Server properties and opaque network configuration offsets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpeedSources {
    pub creature_speed: i16,
    pub config_548: i32,
    /// Resolved selected-record +24, only when the server selectors enable it.
    /// None means the server selectors disable the override, not missing data.
    /// TODO ❓ The table name and selector-to-record lookup are not mapped.
    /// Do not construct these sources when an enabled override is unresolved.
    pub selected_record_24: Option<i16>,
    pub config_552: i32,
    pub config_556: i32,
    pub prior_speed_3904: i32,
    pub prior_age: u64,
    pub upward_enabled_1497: bool,
    pub upward_speed_1500: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpeedResult {
    pub selected_speed: i64,
    pub speed_with_margin: i64,
    /// Expired history is cleared after its old value contributes to this call.
    pub retained_prior_speed: i32,
}

/// Exact source selection. Constants are code values, not game table records.
/// Spec: Step 5 part 1, Speed and duration sources; 381 0x14048E6C0.
pub fn selected_speed(t: MoveType, sources: &SpeedSources) -> Result<i64, Failure> {
    let speed = i64::from(sources.creature_speed);
    Ok(match t.code {
        0 | 1 => sources
            .selected_record_24
            .map(i64::from)
            .unwrap_or_else(|| speed.min(i64::from(sources.config_548))),
        2..=4 => speed,
        14 if !t.flag => speed,
        5..=13 | 14 | 34..=36 => 109,
        15 => 50 * speed / 100,
        16..=18 => 79,
        22..=24 => 100,
        25..=27 => 150 * speed / 100,
        28..=33 | 40..=42 => 94,
        _ => return Err(Failure::Reason(119)),
    })
}

/// Squared XY/upward bounds. Equality passes. No integer-triple telemetry gate.
/// Spec: Step 5 part 1, Speed and duration sources; 381 0x14048E6C0.
/// Run input limits separately. This function does not approve zero-duration moves.
pub fn speed_check(
    start: Position,
    end: Position,
    duration: u32,
    types: &[MoveType],
    sources: &SpeedSources,
) -> Result<SpeedResult, Failure> {
    if types.is_empty() {
        return Err(Failure::Reason(2));
    }
    let mut max_speed = i64::MIN;
    for &t in types {
        if !valid_move_type(t.code) {
            return Err(Failure::Reason(119));
        }
        max_speed = max_speed.max(selected_speed(t, sources)?);
    }
    max_speed = max_speed.max(i64::from(sources.prior_speed_3904));
    let margin = if types.iter().any(|t| matches!(t.code, 0 | 1)) {
        sources.config_556
    } else {
        sources.config_552
    };
    let s = max_speed + i64::from(margin);
    let d = i128::from(duration);
    let dx = ((i128::from(end.x) - i128::from(start.x)).abs() - 1).max(0);
    let dy = ((i128::from(end.y) - i128::from(start.y)).abs() - 1).max(0);
    let dz = i128::from(end.z) - i128::from(start.z);
    let square = |v: i128| v * v;
    if (duration > 0 && square(i128::from(s) * d) < 1_000_000 * (square(dx) + square(dy)))
        || (sources.upward_enabled_1497
            && dz > 0
            && square(i128::from(sources.upward_speed_1500) * d) < 1_000_000 * square(dz))
    {
        return Err(Failure::UnmappedReason(UnmappedGate::Speed));
    }
    Ok(SpeedResult {
        selected_speed: max_speed,
        speed_with_margin: s,
        retained_prior_speed: if sources.prior_age > 500 {
            0
        } else {
            sources.prior_speed_3904
        },
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DurationHistory {
    pub rolling_total: u64,
    pub elapsed_server_ticks: u64,
    pub config_560: i32,
    pub enforce_564: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DurationResult {
    pub excess: bool,
    pub accepted: bool,
}

/// Optional server history: excess is nonfatal when enforcement is disabled.
/// Spec: Step 5 part 1, Speed and duration sources; 381 0x1405D87D0.
/// TODO ❓ Ring initialization, updates and configured units are not mapped.
pub fn duration_history(history: Option<DurationHistory>, request_duration: u32) -> DurationResult {
    let excess = history.is_some_and(|h| {
        i128::from(h.rolling_total) + i128::from(request_duration)
            > i128::from(h.elapsed_server_ticks) + i128::from(h.config_560)
    });
    DurationResult {
        excess,
        accepted: !excess || !history.is_some_and(|h| h.enforce_564),
    }
}
