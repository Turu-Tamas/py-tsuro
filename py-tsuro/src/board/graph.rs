use itertools::Itertools;
use pyo3_tch::*;
use std::iter::zip;
use tch::{Kind, Tensor};

use crate::*;

#[pymethods]
impl MarkerPosition {
    pub fn node_id(&self) -> usize {
        let (mut x, mut y) = self.coords;
        if x % 3 == 0 {
            x /= 3;
            y = y - 1 - (y / 3);
            return x * 12 + y;
        } else {
            y /= 3;
            x = x - 1 - (x / 3);
            return (6 * 12 + 12) + y * 12 + x;
        }
    }
}

impl BoardGraph {
    /// initialize graph for an empty board
    pub(super) fn new() -> Self {
        let mut ret = BoardGraph {
            adjacency_list: [const { vec![] }; 168],
            vertices: [None; 168],
        };

        // all valid positions are nodes of the graph
        for (x, y) in (0..19).cartesian_product(0..19) {
            if (x % 3 == 0) ^ (y % 3 == 0) {
                let markerpos = MarkerPosition::from_lattice_coordinates((x, y));
                let idx = markerpos.node_id();
                ret.vertices[idx] = Some(markerpos);
            }
        }

        // add edges
        for (id, position) in ret.vertices.iter().enumerate() {
            let position = position.unwrap(); // all positions are nodes
            let adjacent_tiles = position.adjacent_tiles();

            for tile_coord in adjacent_tiles.iter() {
                // entry point index of the node
                let idx = position.entry_point_index_on(*tile_coord).unwrap();

                for entry_idx in 0..8 {
                    if entry_idx == idx {
                        continue; // skip the node whose neighbors we are trying to find
                    }
                    let neighbour = MarkerPosition::from_entry_point_index(
                        *tile_coord,
                        entry_idx,
                    );
                    ret.adjacency_list[id].push((neighbour.node_id(), false));
                }
            }
        }
        ret
    }

    fn remove_edge(&mut self, node_ids: (usize, usize)) {
        let (a, b) = node_ids;
        for (from_id, to_id) in [(a, b), (b, a)] {
            let idx = self.adjacency_list[from_id]
                .iter()
                .position(|(other_id, _built)| *other_id == to_id)
                .expect("adjacancy list should contain the id if there exists an edge between the nodes");
            self.adjacency_list[from_id].swap_remove(idx);
        }
    }

    pub(super) fn place_tile(&mut self, tile: Tile, tile_coords: Coord) {
        // remove all connections going across this tile
        // filter this way so each edge is included only once
        let all_connections = (0..8).cartesian_product(0..8).filter(|(a, b)| a < b);
        for (a, b) in all_connections {
            let marker_pos_a =
                MarkerPosition::from_entry_point_index(tile_coords, a);
            let marker_pos_b =
                MarkerPosition::from_entry_point_index(tile_coords, b);
            let node_a = marker_pos_a.node_id();
            let node_b = marker_pos_b.node_id();
            self.remove_edge((node_a, node_b));
        }

        // Create new path connections based on the tile's pattern
        // and connect them to any existing paths
        for entry_indices in tile.paths() {
            let entry_indices = [entry_indices.0, entry_indices.1];

            let mut path_ends = [0, 0];
            for (path_end_id, entry_idx) in
                zip(path_ends.iter_mut(), entry_indices.iter())
            {
                let position =
                    MarkerPosition::from_entry_point_index(tile_coords, *entry_idx);
                let node_id = position.node_id();
                // if this node is the end of a path, it is now only connected with the other end of that path
                // (we already removed the edges going across this tile)
                *path_end_id = if self.adjacency_list[node_id].len() == 1 {
                    self.adjacency_list[node_id][0].0
                } else {
                    node_id
                };

                if *path_end_id != node_id {
                    // path goes through node, node can be removed
                    self.vertices[node_id] = None;
                    self.remove_edge((*path_end_id, node_id));
                }
            }

            self.adjacency_list[path_ends[0]].push((path_ends[1], true));
            self.adjacency_list[path_ends[1]].push((path_ends[0], true));
        }
    }
}

#[pymethods]
impl BoardGraph {
    pub fn adjacency_matrix_tensor<'py>(&self) -> PyTensor {
        let mut adjacency_matrix = vec![vec![0i8; 168]; 168];
        for (from_id, neighbors) in self.adjacency_list.iter().enumerate() {
            for &(to_id, built) in neighbors {
                adjacency_matrix[from_id][to_id] = if built { 1 } else { -1 };
            }
        }

        let adjacency_matrix_tensor = Tensor::from_slice(&adjacency_matrix.concat())
            .view([168, 168])
            .to_kind(Kind::Int8);

        PyTensor(adjacency_matrix_tensor)
    }
}

#[test]
fn test_graph() {
    let mut graph = BoardGraph::new();

    graph.place_tile(find_tile_with_connection(0, 5), (0, 0));
    let start = MarkerPosition::from_entry_point_index((0, 0), 5).node_id();
    let end = MarkerPosition::from_entry_point_index((0, 0), 0).node_id();

    assert_eq!(graph.adjacency_list[start], vec![(end, true)]);

    assert!(graph.adjacency_list[end].contains(&(start, true)));
    for entry_idx in 0..8 {
        if entry_idx == 5 {
            continue;
        }
        let pos = MarkerPosition::from_entry_point_index((0, 1), entry_idx);
        let node_id = pos.node_id();
        assert!(graph.adjacency_list[end].contains(&(node_id, false)));
        assert!(graph.adjacency_list[node_id].contains(&(end, false)));
    }
    assert_eq!(graph.adjacency_list[end].len(), 8);

    graph.place_tile(find_tile_with_connection(0, 5), (0, 1));
    assert_eq!(graph.vertices[end], None);
    assert_eq!(graph.adjacency_list[end], vec![]);
    let end = MarkerPosition::from_entry_point_index((0, 1), 0).node_id();
    assert!(graph.adjacency_list[end].contains(&(start, true)));
}
