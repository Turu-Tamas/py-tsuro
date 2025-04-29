use std::cell::LazyCell;

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

mod board;
mod env;
mod pymethods;
mod tile;
mod view;

#[cfg(test)]
pub(crate) use tile::find_tile_with_connection;
pub(crate) type Coord = (usize, usize);
pub use tile::ALL_TILES;

#[pyclass(module = "py_tsuro")]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Board {
    pub markers: Vec<Option<Marker>>,
    #[pyo3(get)]
    pub tiles: [[Option<Tile>; 6]; 6],
    #[pyo3(get)]
    pub graph: BoardGraph,
}

#[pyclass(module = "py_tsuro")]
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Default,
)]
pub struct Tile {
    #[pyo3(get)]
    pub connections: [usize; 8],
}

#[pyclass(module = "py_tsuro")]
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct View {
    #[pyo3(get)]
    pub board: Board,
    #[pyo3(get)]
    pub hand: Vec<Tile>,
    #[pyo3(get)]
    pub active_player: usize,
}

#[pyclass(module = "py_tsuro")]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BoardGraph {
    #[pyo3(get)]
    #[serde(with = "BigArray")]
    pub vertices: [Option<MarkerPosition>; 168],
    /// adjacency_list[from_id] == [(to_id, built)]
    /// to_id: id of the other node
    /// built: wether the path is built
    #[pyo3(get)]
    #[serde(with = "BigArray")]
    pub adjacency_list: [Vec<(usize, bool)>; 168],
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[pyclass(module = "py_tsuro", subclass)]
pub struct TsuroEnv {
    board: Board,
    num_markers_placed: usize,
    phase: Phase,
    num_players: usize,
    active_player: usize,
    player_hands: Vec<Vec<Tile>>,
    deck: Vec<Tile>,
    dragon_tile_owner: Option<usize>,
    num_players_left: usize,
}

#[pyclass(module = "py_tsuro")]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Serialize, Deserialize,
)]
pub struct Marker {
    #[pyo3(get)]
    pub position: MarkerPosition,
    pub(crate) previous_tile: Option<Coord>,
    pub(crate) has_moved: bool,
}

/// marker positions can be understood as coordinates on the 19x19 lattice
/// created by splitting each tile in 3x3 subtiles,
/// or as a 6x6 coordinate plus the entry point index
/// entry point indices start from 0 at the left point on the south side
/// this struct provides methods to view and manage it as either
#[pyclass(module = "py_tsuro", name = "MarkerPosition")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MarkerPosition {
    /// 19x19 lattice coordinates
    #[pyo3(get)]
    pub coords: Coord,
}

#[pyclass(module = "py_tsuro")]
#[derive(PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Phase {
    #[default]
    Markers = 0,
    Tiles = 1,
}

#[pyclass(module = "py_tsuro")]
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct EnvReturn {
    #[pyo3(get)]
    pub view: View,
    #[pyo3(get)]
    pub terminated: bool,
    #[pyo3(get)]
    pub move_is_valid: bool,
    #[pyo3(get)]
    pub active_player: usize,
    #[pyo3(get)]
    pub remaining_players: Vec<usize>,
    #[pyo3(get)]
    pub phase: Phase,
}

pub const ALL_NODES: LazyCell<Vec<MarkerPosition>> = LazyCell::new(|| {
    let mut out = Vec::with_capacity(168);
    for x in 0..19 {
        for y in 0..19 {
            if (x % 3 == 0) ^ (y % 3 == 0) {
                out.push(MarkerPosition::from_lattice_coordinates((x, y)));
            }
        }
    }
    out
});

#[pymodule]
fn py_tsuro<'py>(py: Python<'py>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    py.import("torch")?;
    m.add_class::<Tile>()?;
    m.add_class::<EnvReturn>()?;
    m.add_class::<View>()?;
    m.add_class::<TsuroEnv>()?;
    m.add_class::<MarkerPosition>()?;
    m.add_class::<Phase>()?;
    m.add_class::<BoardGraph>()?;
    m.add_class::<Board>()?;

    #[allow(clippy::borrow_interior_mutable_const)]
    m.add("ALL_TILES", *ALL_TILES)?;
    m.add("ALL_NODES", ALL_NODES.clone())
}
