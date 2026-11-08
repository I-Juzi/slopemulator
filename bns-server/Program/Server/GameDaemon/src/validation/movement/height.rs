use super::{Failure, Layer};

/// All inputs are server state or resolved cells, except the two client heights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeightInput {
    pub native_kind: u8,
    pub server_state_2300: u8,
    pub origin: Layer,
    pub next: Layer,
    pub client_origin_z: i16,
    pub client_next_z: i16,
    pub move_type: i8,
    pub short_2256: i16,
    pub glide_2288: i32,
    pub boost_2292: i32,
    pub dive_2296: i32,
    /// TODO ❓ Initialization/value of global 381 0x140E7BCF8 is not mapped.
    pub jump_global: Option<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeightPending {
    JumpGlobal,
    JumpTrackingOrEndJump,
    RisingFallHelper,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeightResult {
    Accepted,
    Change {
        state: u8,
        at_z: i32,
    },
    /// Only the new state is mapped. TODO ❓ Other transition effects are open.
    StateOnly {
        state: u8,
    },
    /// Not approval. The caller must resolve the missing rule before continuing.
    Pending(HeightPending),
}

/// Exact height/state subset, with explicit stops at unmapped branches.
/// Spec: Step 5 part 1, Height/state rules; 381 0x14048CDB0.
/// TODO ❓ Jump tracking/end-jump actions and 0x14048BF70 are not mapped.
/// States 11/12 and unknown states pass only this helper, not all movement.
pub fn height_state(i: HeightInput) -> Result<HeightResult, Failure> {
    use HeightResult::{Accepted, Change, Pending, StateOnly};
    if i.native_kind != 1 {
        return Err(Failure::Reason(100));
    }
    let z0 = i32::from(i.client_origin_z);
    let z1 = i32::from(i.client_next_z);
    let f1 = i32::from(i.next.floor);
    let h = z1 - f1;
    let delta = f1 - i32::from(i.origin.floor);
    let band = |low: i32, high: i32, attribute: u8, reason: i16| {
        if delta > high {
            Err(Failure::Reason(112))
        } else if delta < low {
            Err(Failure::Reason(100))
        } else if i.next.attribute_11 != attribute {
            Err(Failure::Reason(reason))
        } else {
            Ok(Accepted)
        }
    };
    match i.server_state_2300 {
        0 => {
            if h > 11 {
                Ok(Change { state: 2, at_z: z0 })
            } else {
                band(-24, 20, 0, 144)
            }
        }
        1 => {
            let Some(global) = i.jump_global else {
                return Ok(Pending(HeightPending::JumpGlobal));
            };
            if i64::from(z1) <= i64::from(i.short_2256) + i64::from(global) {
                Ok(Pending(HeightPending::JumpTrackingOrEndJump))
            } else if h <= 11 {
                Ok(StateOnly {
                    state: if i.next.attribute_11 == 1 { 8 } else { 0 },
                })
            } else {
                Err(Failure::Reason(113))
            }
        }
        2 => {
            if i.move_type == 14 {
                return Ok(Accepted);
            }
            if z1 - z0 < -1000 {
                return Err(Failure::Reason(115));
            }
            if z1 > z0 && h > 16 {
                return Ok(Pending(HeightPending::RisingFallHelper));
            }
            if h < 10 {
                Ok(Change {
                    state: if i.next.attribute_11 != 0 { 8 } else { 0 },
                    at_z: f1,
                })
            } else {
                Ok(Accepted)
            }
        }
        3 => {
            if h < 11 {
                Ok(Change { state: 0, at_z: f1 })
            } else {
                Ok(Accepted)
            }
        }
        4 => {
            if h <= 22 {
                band(-24, 20, 0, 144)
            } else if z1 <= z0 {
                Ok(Accepted)
            } else {
                Err(Failure::Reason(100))
            }
        }
        5 => {
            if z1 <= i32::from(i.short_2256) + 135 {
                Ok(Accepted)
            } else {
                Err(Failure::Reason(113))
            }
        }
        6 | 8 => {
            if h > 11 {
                Ok(Change { state: 2, at_z: z0 })
            } else {
                band(-15, 15, 1, 138)
            }
        }
        7 => {
            if i64::from(z1) <= i64::from(i.glide_2288) + 25 {
                Ok(Accepted)
            } else {
                Err(Failure::Reason(146))
            }
        }
        9 => {
            if i64::from(z1) <= i64::from(i.boost_2292) + 5 {
                Ok(Accepted)
            } else {
                Err(Failure::Reason(153))
            }
        }
        10 => {
            if z1 <= i.dive_2296 {
                Ok(Accepted)
            } else {
                Err(Failure::Reason(154))
            }
        }
        _ => Ok(Accepted),
    }
}

/// Apply an externally resolved fallback result, not an invented fallback rule.
/// Spec: Step 5 part 1, Height/state rules, state 2; 381 0x14048CDB0,
/// 0x14048BF70. TODO ❓ The fallback predicate and other effects are not mapped.
pub fn rising_fall_result(accepted: bool) -> Result<HeightResult, Failure> {
    if accepted {
        Ok(HeightResult::StateOnly { state: 3 })
    } else {
        Err(Failure::Reason(114))
    }
}
