mod edge_index_to_position;

use crate::*;
use arrayvec::ArrayVec;
use edge_index_to_position::INDEX_TO_POSITION;

impl MarkerPosition {
    pub fn from_index(idx: usize) -> Self {
        MarkerPosition {
            #[allow(clippy::borrow_interior_mutable_const)]
            coords: INDEX_TO_POSITION[idx],
        }
    }

    pub fn from_entry_point_index(position: Coord, entry_point: usize) -> Self {
        let (x, y) = position;
        let lattice_x = x * 3
            + match entry_point {
                6 | 7 => 0,
                0 | 5 => 1,
                1 | 4 => 2,
                2 | 3 => 3,
                _ => panic!("Invalid entry point"),
            };
        let lattice_y = y * 3
            + match entry_point {
                4 | 5 => 0,
                3 | 6 => 1,
                2 | 7 => 2,
                0 | 1 => 3,
                _ => panic!("Invalid entry point"),
            };
        Self {
            coords: (lattice_x, lattice_y),
        }
    }

    pub fn from_lattice_coordinates(position: Coord) -> Self {
        Self { coords: position }
    }

    /// adjacent coordinates, with corresponding entry point indices
    /// if there is only one, it returns that twice
    pub fn entry_point_indices(&self) -> ArrayVec<(Coord, usize), 2> {
        let (x, y) = self.coords;

        if x == 18 {
            let local_y = y % 3;
            let entry_point = match local_y {
                1 => 3,
                2 => 2,
                _ => panic!("Invalid lattice coordinates"),
            };
            let ret = ((5, y / 3), entry_point);
            return ArrayVec::from_iter([ret]);
        }
        if y == 18 {
            let local_x = x % 3;
            let entry_point = match local_x {
                1 => 0,
                2 => 1,
                _ => panic!("Invalid lattice coordinates"),
            };
            let ret = ((x / 3, 5), entry_point);
            return ArrayVec::from_iter([ret]);
        }

        let tile_x = x / 3;
        let tile_y = y / 3;
        let local_x = x % 3;
        let local_y = y % 3;

        let entry_point = match (local_x, local_y) {
            (2, 0) => 4,
            (1, 0) => 5,
            (0, 1) => 6,
            (0, 2) => 7,
            _ => panic!("Invalid lattice coordinates"),
        };

        if x == 0 || y == 0 {
            let ret = ((tile_x, tile_y), entry_point);
            return ArrayVec::from_iter([ret]);
        }

        let adjacent_tile = match (local_x, local_y) {
            (0, _) => (tile_x.saturating_sub(1), tile_y),
            (_, 0) => (tile_x, tile_y.saturating_sub(1)),
            _ => panic!("Invalid lattice coordinates"),
        };

        // the one exactly opposite
        let adjacent_entry_point = match entry_point {
            5 => 0,
            4 => 1,
            7 => 2,
            6 => 3,
            _ => panic!(),
        };

        ArrayVec::from([
            ((tile_x, tile_y), entry_point),
            (adjacent_tile, adjacent_entry_point),
        ])
    }

    pub fn entry_point_index_on(&self, pos: Coord) -> Option<usize> {
        let (positions, entry_indices): (Vec<_>, Vec<_>) =
            self.entry_point_indices().into_iter().unzip();
        if pos == positions[0] {
            return Some(entry_indices[0]);
        } else if pos == positions[1] {
            return Some(entry_indices[1]);
        } else {
            return None;
        }
    }

    pub fn adjacent_tiles(&self) -> ArrayVec<Coord, 2> {
        self.entry_point_indices()
            .into_iter()
            .map(|(pos, _entry_idx)| pos)
            .collect()
    }
}

#[pymethods]
impl MarkerPosition {
    pub fn is_edge(&self) -> bool {
        matches!(self.coords, (0, _) | (18, _) | (_, 0) | (_, 18))
    }
}

impl Default for MarkerPosition {
    fn default() -> Self {
        MarkerPosition {
            coords: (usize::MAX, usize::MAX),
        }
    }
}

#[test]
fn test_from_entry_point_indices() {
    let tests = [
        (3, 4),
        (5, 3),
        (6, 4),
        (5, 6),
        (0, 1),
        (2, 18),
        (18, 13),
        (16, 9),
    ];
    let answers = [
        [((1, 1), 6), ((0, 1), 3)].as_slice(),
        [((1, 1), 4), ((1, 0), 1)].as_slice(),
        [((2, 1), 6), ((1, 1), 3)].as_slice(),
        [((1, 2), 4), ((1, 1), 1)].as_slice(),
        [((0, 0), 6)].as_slice(),
        [((0, 5), 1)].as_slice(),
        [((5, 4), 3)].as_slice(),
        [((5, 3), 5), ((5, 2), 0)].as_slice(),
    ];
    for (test, answer) in tests.into_iter().zip(answers.into_iter()) {
        let mp = MarkerPosition { coords: test };
        assert_eq!(*mp.entry_point_indices(), *answer)
    }
}
