use pyo3::pymethods;
use std::cell::LazyCell;

use crate::*;

#[pymethods]
impl Tile {
    pub fn rotated(&self, mut num: usize) -> Self {
        num %= 4;
        let mut new_conn = <[usize; 8]>::default();
        for (i, val) in new_conn.iter_mut().enumerate() {
            // index of the point that will move to the ith
            let idx = if i < num * 2 {
                8 + i - num * 2
            } else {
                i - num * 2
            };
            *val = (self.connections[idx] + num * 2) % 8;
        }
        Tile {
            connections: new_conn,
        }
    }

    pub fn paths(&self) -> [(usize, usize); 4] {
        let mut used_vertices = [false; 8];
        let mut out = [(0, 0); 4];
        let mut out_idx = 0;
        for (a, b) in self.connections.into_iter().enumerate() {
            if used_vertices[a] || used_vertices[b] {
                continue;
            }
            used_vertices[a] = true;
            used_vertices[b] = true;
            out[out_idx] = (a, b);
            out_idx += 1;
        }
        out
    }
}

#[test]
fn test_rotate_tile() {
    let tile = Tile {
        connections: [1, 0, 5, 7, 6, 2, 4, 3],
    };
    assert_eq!(tile.rotated(1).connections, [6, 5, 3, 2, 7, 1, 0, 4]);
    assert_eq!(tile.rotated(2).connections, [2, 6, 0, 7, 5, 4, 1, 3]);
    assert_eq!(tile.rotated(3).connections, [3, 5, 4, 0, 2, 1, 7, 6]);
    assert_eq!(tile.rotated(2), tile.rotated(6));
    assert_eq!(tile.rotated(1).rotated(1), tile.rotated(2));
    assert_eq!(tile, tile.rotated(0))
}

impl Tile {
    fn new(code: &str) -> Self {
        let s = code.split('-');
        let mut connections: [usize; 8] = [0; 8];
        for connection in s {
            let mut c = connection.bytes();
            let a = c.next().unwrap();
            let b = c.next().unwrap();
            let from = a - b'1';
            let to = b - b'1';
            connections[from as usize] = to as usize;
            connections[to as usize] = from as usize;
        }
        Tile { connections }
    }
}

#[allow(clippy::declare_interior_mutable_const)]
pub const ALL_TILES: LazyCell<[Tile; 35]> = LazyCell::new(|| {
    [
        Tile::new("12-34-56-78"),
        Tile::new("14-27-36-58"),
        Tile::new("15-26-37-48"),
        Tile::new("16-25-38-47"),
        Tile::new("18-23-45-67"),
        Tile::new("12-37-48-56"),
        Tile::new("12-38-47-56"),
        Tile::new("16-25-37-48"),
        Tile::new("17-24-35-68"),
        Tile::new("15-27-36-48"),
        Tile::new("17-28-35-46"),
        Tile::new("18-26-37-45"),
        Tile::new("18-27-36-45"),
        Tile::new("13-26-48-57"),
        Tile::new("15-28-37-46"),
        Tile::new("12-35-47-68"),
        Tile::new("12-36-47-58"),
        Tile::new("12-38-45-67"),
        Tile::new("12-38-46-57"),
        Tile::new("17-24-36-58"),
        Tile::new("18-23-46-57"),
        Tile::new("12-34-57-68"),
        Tile::new("12-34-58-67"),
        Tile::new("16-23-47-58"),
        Tile::new("16-28-35-47"),
        Tile::new("17-23-46-58"),
        Tile::new("17-28-36-45"),
        Tile::new("12-36-48-57"),
        Tile::new("12-37-46-58"),
        Tile::new("12-37-45-68"),
        Tile::new("12-35-48-67"),
        Tile::new("13-26-47-58"),
        Tile::new("15-28-36-47"),
        Tile::new("13-25-48-67"),
        Tile::new("16-28-37-45"),
    ]
});

#[cfg(test)]
pub fn find_tile_with_connection(from: usize, to: usize) -> Tile {
    ALL_TILES
        .iter()
        .flat_map(|tile| (0..4).map(|rotation| tile.rotated(rotation)))
        .filter(|tile| tile.connections[from] == to)
        .next()
        .unwrap()
}
