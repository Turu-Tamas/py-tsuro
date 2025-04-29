use std::vec;

use itertools::Itertools;

#[cfg(test)]
use crate::find_tile_with_connection;

use crate::*;

mod graph;
mod marker;

impl Board {
    pub fn new() -> Self {
        Self {
            markers: vec![],
            tiles: [[None; 6]; 6],
            graph: BoardGraph::new(),
        }
    }
}

impl Board {
    pub fn place_marker(&mut self, position_index: usize) -> bool {
        let marker = Some(Marker {
            position: MarkerPosition::from_index(position_index),
            previous_tile: None,
            has_moved: false,
        });
        if self.markers.contains(&marker) {
            return false;
        }
        self.markers.push(marker);
        true
    }

    pub fn place_tile(&mut self, tile: Tile, player: usize) {
        let (x, y) = self.next_tile_of_player(player);
        let old_tile = &mut self.tiles[x][y];
        assert!(old_tile.is_none());
        self.graph.place_tile(tile, (x, y));
        *old_tile = Some(tile);
    }

    /// returns the players that would collide if the tile was placed in the position
    /// works if the tile has already been placed provided the pawns have not been moved yet
    pub fn find_collisions(&self, tile: Tile, position: Coord) -> Vec<usize> {
        let mut collisions = Vec::new();
        let (x, y) = position;

        // The positions on the edge of the tile
        let positions = (0..8)
            .map(|idx| MarkerPosition::from_entry_point_index((x, y), idx))
            .collect_vec();

        let marker_positions = self
            .markers
            .iter()
            .map(|marker| marker.map(|marker| marker.position))
            .collect_vec();

        // Check if two players have pawns on two ends of a path on the newly placed tile
        for (entry_idx_a, entry_idx_b) in tile.paths() {
            let player_a = marker_positions
                .iter()
                .position(|pos| *pos == Some(positions[entry_idx_a]));
            let player_b = marker_positions
                .iter()
                .position(|pos| *pos == Some(positions[entry_idx_b]));

            if let (Some(player_a), Some(player_b)) = (player_a, player_b) {
                collisions.push(player_a);
                collisions.push(player_b);
            }
        }

        // Remove duplicates
        collisions.sort_unstable();
        collisions.dedup();
        collisions
    }

    /// returns list of players that should be eliminated (reached the edge)
    /// eliminated players' positions are not updated
    pub fn move_markers(&mut self) -> Vec<usize> {
        let mut eliminated = vec![];
        for (player, marker) in self.markers.clone().iter().enumerate() {
            if marker.is_none() {
                continue; // eliminated player
            }
            let (new_pos, is_different) = self.find_player_path_end(player);

            if !is_different {
                continue; // player does not move, nothing to do
            }

            if new_pos.is_edge() {
                eliminated.push(player);
            } else {
                // new position is either at the edge of the board,
                // or next to an empty tile
                let (a, b) = new_pos
                    .adjacent_tiles()
                    .into_iter()
                    .collect_tuple()
                    .unwrap();
                // the previous one is the one that is not None
                let prev_tile_pos =
                    if self.tiles[a.0][a.1].is_none() { b } else { a };
                assert!(self.tiles[prev_tile_pos.0][prev_tile_pos.1].is_some());
                self.markers[player] = Some(Marker {
                    previous_tile: Some(prev_tile_pos),
                    position: new_pos,
                    has_moved: true,
                });
            }
        }
        eliminated
    }

    pub fn move_is_suicide(&self, tile: Tile, active_player: usize) -> bool {
        let marker =
            self.markers[active_player].expect("player should not be eliminated");
        let tile_pos = self.next_tile_of_player(active_player);

        // the positions on this tile where the path does not end
        // it ends on positions adjacent to an empty tile
        // (but not the one we would be replacing)
        let nonterminal_positions = (0..8)
            .map(|entry_point| {
                MarkerPosition::from_entry_point_index(tile_pos, entry_point)
            })
            .filter(|position| {
                let adjacents = position.adjacent_tiles();
                adjacents.len() == 2
                    && adjacents.into_iter().all(|(x, y)| {
                        self.tiles[x][y].is_some() || (x, y) == tile_pos
                    })
            })
            .collect_vec();

        let mut current_position = marker.position;
        // follow path until it ends on a tile other than the one we would place
        // because find_path_endpoint does not see this tile and thinks the path ends here
        loop {
            let entry_point =
                current_position.entry_point_index_on(tile_pos).unwrap();
            let exit_point = tile.connections[entry_point];
            current_position = self.find_path_endpoint(tile_pos, exit_point);
            if !nonterminal_positions.contains(&current_position) {
                break;
            }
        }

        let end = current_position;
        let colliders = self.find_collisions(tile, tile_pos);

        end.is_edge() || colliders.contains(&active_player)
    }

    pub fn eliminate_player(&mut self, player: usize) {
        self.markers[player] = None;
    }

    /// next tile this player will move to
    pub fn next_tile_of_player(&self, player: usize) -> Coord {
        let marker = self.markers[player].expect(
            "next_tile_position_of called on a player \
            that has been eliminated or has not yet placed a marker",
        );
        let adjacents = marker.position.adjacent_tiles();
        match marker.previous_tile {
            Some(previous) => {
                // the player has moved and has not been eliminated
                assert!(adjacents.len() == 2);
                if adjacents[0] == previous {
                    adjacents[1]
                } else {
                    adjacents[0]
                }
            }
            None => {
                assert!(marker.position.is_edge());
                // only one adjacent because marker has not moved
                adjacents[0]
            }
        }
    }
}

impl Board {
    /// end of the path the player is on, and wether it is different from the current position
    fn find_player_path_end(&self, player: usize) -> (MarkerPosition, bool) {
        let marker = self.markers[player]
            .expect("should not be called with a player that has been eliminated");

        // TODO this could definitely be better (use next_tile_of_player)
        let coords;
        let exit_idx;
        if marker.previous_tile.is_none() {
            // special case where the marker is still on the edge of the board
            assert!(marker.position.is_edge());
            let (x, y) = marker.position.adjacent_tiles()[0]; // next tile
            if self.tiles[x][y].is_none() {
                return (marker.position, false);
            }
            let tile = self.tiles[x][y].unwrap();
            let from_idx = marker.position.entry_point_index_on((x, y)).unwrap();
            let to_idx = tile.connections[from_idx];
            coords = (x, y);
            exit_idx = to_idx;
        } else {
            coords = marker.previous_tile.unwrap();
            exit_idx = marker.position.entry_point_index_on(coords).unwrap();
        }

        let end = self.find_path_endpoint(coords, exit_idx);
        (end, end != marker.position)
    }

    /// the position following the path after exiting the
    /// tile at location at the point indicated by exit
    pub(crate) fn find_path_endpoint(
        &self,
        location: Coord,
        exit: usize,
    ) -> MarkerPosition {
        let mut current_position =
            MarkerPosition::from_entry_point_index(location, exit);
        let mut last_tile_coord = location;
        loop {
            let adjacents = current_position.adjacent_tiles();
            let next_tile_coord =
                if adjacents.len() == 2 && last_tile_coord == adjacents[0] {
                    adjacents[1]
                } else {
                    adjacents[0]
                };
            if next_tile_coord == last_tile_coord {
                // occurs when there is only one adjacent tile,
                // in which case the path ends here
                break;
            }
            let next_tile = self.tiles[next_tile_coord.0][next_tile_coord.1];
            if next_tile.is_none() {
                break;
            }
            let entry_index = current_position
                .entry_point_index_on(next_tile_coord)
                .unwrap();
            let exit_index = next_tile.unwrap().connections[entry_index];
            last_tile_coord = next_tile_coord;
            current_position =
                MarkerPosition::from_entry_point_index(next_tile_coord, exit_index);
        }
        current_position
    }
}

#[test]
fn test_path_end_of() {
    let mut board = Board::new();
    board.tiles[4][1] = Some(Tile {
        connections: [7, 69, 5, 69, 69, 2, 69, 0],
    });
    board.tiles[5][1] = Some(find_tile_with_connection(7, 1));
    board.tiles[5][2] = Some(Tile {
        connections: [6, 4, 69, 69, 1, 69, 0, 69],
    });
    board.tiles[5][3] = Some(find_tile_with_connection(4, 5));
    board.tiles[4][2] = Some(find_tile_with_connection(3, 5));
    board.tiles[3][1] = Some(find_tile_with_connection(2, 4));
    board.tiles[3][0] = Some(find_tile_with_connection(1, 4));
    assert_eq!(
        board.find_path_endpoint((4, 0), 0),
        MarkerPosition::from_lattice_coordinates((11, 0))
    );
}

#[test]
fn test_move_markers() {
    let mut board = Board::new();
    board.place_marker(28);
    board.tiles[3][0] = Some(find_tile_with_connection(4, 1));
    board.move_markers();
    assert_eq!(
        board.markers[0].unwrap().position,
        MarkerPosition::from_lattice_coordinates((11, 3))
    );
    board.tiles[3][1] = Some(find_tile_with_connection(4, 2));
    board.move_markers();
    assert_eq!(
        board.markers[0].unwrap().position,
        MarkerPosition::from_lattice_coordinates((12, 5))
    );
    board.tiles[4][1] = Some(find_tile_with_connection(7, 2));
    board.tiles[5][1] = Some(find_tile_with_connection(7, 5));
    board.move_markers();
    assert_eq!(
        board.markers[0].unwrap().position,
        MarkerPosition::from_lattice_coordinates((16, 3))
    );
    board.tiles[5][0] = Some(find_tile_with_connection(0, 6));
    board.tiles[4][0] = Some(find_tile_with_connection(3, 4));
    assert!(board.move_markers() == vec![0]);
}
