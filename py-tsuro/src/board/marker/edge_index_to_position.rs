use super::Coord;
use std::cell::LazyCell;

/// Convert from an index on the edge of the board to a position
/// 0 is the bottom left corner -> (1; 18)
#[allow(clippy::declare_interior_mutable_const)]
pub const INDEX_TO_POSITION: LazyCell<[Coord; 48]> = LazyCell::new(|| {
    let mut out = [(usize::MAX, usize::MAX); 48];
    let mut idx = 0;
    // y = 18
    for x in 0..18 {
        if x % 3 == 0 {
            continue;
        }
        out[idx] = (x, 18);
        idx += 1;
    }
    // x = 18
    for y_inv in 0..18 {
        if y_inv % 3 == 0 {
            continue;
        }
        out[idx] = (18, 18 - y_inv);
        idx += 1;
    }
    // y = 0
    for x_inv in 0..18 {
        if x_inv % 3 == 0 {
            continue;
        }
        out[idx] = (18 - x_inv, 0);
        idx += 1;
    }
    // x = 0
    for y in 0..18 {
        if y % 3 == 0 {
            continue;
        }
        out[idx] = (0, y);
        idx += 1;
    }
    out
});
