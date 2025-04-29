use itertools::Itertools;
use pyo3::prelude::*;
use std::collections::VecDeque;

use crate::*;

#[pymethods]
impl BoardGraph {
    /// run a bfs from node_id and return a list of (dist, node)
    /// dist: distance from node_id
    /// node: next node id
    pub fn bfs_from(&self, node_id: usize) -> Vec<(usize, usize)> {
        let mut out = Vec::with_capacity(self.vertices.len());
        let mut queue = VecDeque::new();
        let mut visited = [false; 168];

        queue.push_back((node_id, 0));
        visited[node_id] = true;
        out.push((0, node_id));

        while !queue.is_empty() {
            let (current_id, current_dist) = queue.pop_front().unwrap();
            let next_dist = current_dist + 1;

            for (to_id, _built) in &self.adjacency_list[current_id] {
                if visited[*to_id] {
                    continue;
                }
                visited[*to_id] = true;
                queue.push_back((*to_id, next_dist));
                out.push((next_dist, *to_id));
            }
        }
        out
    }
}

#[pymethods]
impl View {
    pub fn all_rotated_tiles(&self) -> Vec<Tile> {
        self.hand
            .iter()
            .flat_map(|tile| (0..4).map(|rot| tile.rotated(rot)))
            .sorted_unstable()
            .dedup() // remove duplicates
            .collect()
    }

    pub fn afterstates(&self) -> Vec<(Tile, Board)> {
        let mut out = vec![];
        let tiles = self.all_rotated_tiles();
        let is_suicide: Vec<_> = tiles
            .iter()
            .map(|tile| self.board.move_is_suicide(*tile, self.active_player))
            .collect();
        let all_suicide = is_suicide.iter().all(|x| *x);
        for (tile, suicide) in tiles.into_iter().zip(is_suicide) {
            if suicide && !all_suicide {
                continue;
            }
            let mut board = self.board.clone();
            board.place_tile(tile, self.active_player);
            let eliminated = board.move_markers();
            for player in eliminated {
                board.eliminate_player(player);
            }
            out.push((tile, board));
        }

        out
    }
}
