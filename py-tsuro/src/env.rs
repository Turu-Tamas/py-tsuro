use crate::*;

use itertools::Itertools;
use rand::{seq::SliceRandom, rng};

#[pymethods]
impl TsuroEnv {
    /// takes Option<usize> so calling __new__() works.
    /// required because of unpickling
    #[new]
    pub fn new(num_players: Option<usize>) -> Self {
        if num_players.is_none() {
            return Self::default();
        }
        let num_players = num_players.unwrap();
        #[allow(clippy::borrow_interior_mutable_const)]
        let mut deck = ALL_TILES.into_iter().collect_vec();
        deck.shuffle(&mut rng());
        let mut player_hands = vec![Vec::new(); num_players];
        for hand in player_hands.iter_mut() {
            hand.append(&mut deck.split_off(deck.len() - 3));
        }
        TsuroEnv {
            board: Board::new(),
            num_markers_placed: 0,
            phase: Phase::Markers,
            active_player: 0,
            player_hands,
            deck,
            dragon_tile_owner: None,
            num_players_left: num_players,
            num_players,
        }
    }

    pub fn reset(&mut self) -> EnvReturn {
        *self = Self::new(Some(self.num_players));
        self.get_return(true)
    }

    pub fn step_place_marker(&mut self, position_index: usize) -> EnvReturn {
        if self.num_markers_placed >= self.num_players {
            panic!("cannot place marker, all markers have already been placed");
        }
        let valid = self.board.place_marker(position_index);
        if valid {
            self.num_markers_placed += 1;
        }
        self.end_turn(valid)
    }

    pub fn step_place_tile(&mut self, tile: Tile) -> EnvReturn {
        if self.num_markers_placed != self.num_players {
            panic!("tried to place tile, but not all markers placed");
        }
        if self.terminated() {
            panic!("cannot place tile, game has terminated");
        }
        if !self.move_is_allowed(tile) {
            println!("disallowed");
            return self.end_turn(false);
        }

        self.handle_collisions(tile); // before place_tile
        if self.terminated() {
            return self.end_turn(true);
        }
        self.place_tile(tile);
        self.move_markers(); // also eliminates players
        if !self.terminated() {
            self.draw_tiles();
        }
        self.end_turn(true)
    }

    pub fn get_deck(&self) -> Vec<Tile> {
        self.deck.clone()
    }

    pub fn set_top_tile(&mut self, tile: Tile) {
        let idx = self.deck.iter().position(|t| *t == tile);
        if idx.is_none() {
            panic!("tile not found in deck");
        }
        let idx = idx.unwrap();
        self.deck.swap_remove(idx);
        self.deck.push(tile);
    }

    pub fn clone(&self) -> Self {
        Self {
            board: self.board.clone(),
            num_markers_placed: self.num_markers_placed,
            phase: self.phase,
            active_player: self.active_player,
            player_hands: self.player_hands.clone(),
            deck: self.deck.clone(),
            dragon_tile_owner: self.dragon_tile_owner,
            num_players_left: self.num_players_left,
            num_players: self.num_players,
        }
    }
}

impl TsuroEnv {
    fn end_turn(&mut self, move_is_valid: bool) -> EnvReturn {
        if move_is_valid && !self.terminated() {
            self.active_player = self.player_after(self.active_player);
        }
        if self.phase == Phase::Markers
            && self.num_markers_placed == self.num_players
        {
            self.phase = Phase::Tiles;
        }
        self.get_return(move_is_valid)
    }

    fn place_tile(&mut self, tile: Tile) {
        let tile_idx = self.active_player_hand_find_tile(tile);
        let tile_idx = tile_idx.expect(
            "should not be called unless the move is valid, \
            in which case the player should hold the tile",
        );
        self.player_hands[self.active_player].swap_remove(tile_idx);
        self.board.place_tile(tile, self.active_player);
    }

    fn terminated(&self) -> bool {
        self.num_players_left < 2
            || (self.player_hands.iter().all(|hand| hand.is_empty())
                && self.deck.is_empty())
    }

    /// eliminate players that collide
    fn handle_collisions(&mut self, tile: Tile) {
        let colliding_players = self.board.find_collisions(
            tile,
            self.board.next_tile_of_player(self.active_player),
        );

        for player in colliding_players {
            self.eliminate_player(player);
        }
    }

    /// update self.markers and eliminate players that reached the edge
    fn move_markers(&mut self) {
        let eliminated = self.board.move_markers();
        for player in eliminated {
            self.eliminate_player(player);
        }
    }

    fn draw_tiles(&mut self) {
        let mut current = if self.dragon_tile_owner.is_some() {
            self.dragon_tile_owner.unwrap()
        } else {
            self.active_player
        };

        if !self.deck.is_empty() && self.dragon_tile_owner.is_some() {
            self.dragon_tile_owner = None;
        }

        while !self.deck.is_empty() && self.player_hands[current].len() < 3 {
            self.player_hands[current].push(self.deck.pop().unwrap());
            current = self.player_after(current);
        }

        if self.dragon_tile_owner.is_none() && self.player_hands[current].len() < 3 {
            self.dragon_tile_owner = Some(current);
        }
    }
}

impl TsuroEnv {
    fn view_of(&self, player: usize) -> View {
        View {
            board: self.board.clone(),
            hand: self.player_hands[player].clone(),
            active_player: self.active_player,
        }
    }

    fn player_after(&self, player: usize) -> usize {
        let mut ret = player;
        ret += 1;
        ret %= self.num_players;

        if self.phase == Phase::Markers {
            return ret;
        }

        let mut num_iterations = 0;
        while self.board.markers[ret].is_none() {
            if num_iterations > self.num_players {
                panic!("all players eliminated");
            }
            ret += 1;
            ret %= self.num_players;
            num_iterations += 1;
        }
        if ret == player {
            panic!("only one player left");
        }
        ret
    }

    fn get_return(&self, move_is_valid: bool) -> EnvReturn {
        let remaining_players = if self.num_markers_placed == self.num_players {
            self.board
                .markers
                .iter()
                .enumerate()
                .filter_map(|(player, marker)| marker.map(|_| player))
                .collect_vec()
        } else {
            (0..self.num_players).collect()
        };
        EnvReturn {
            view: self.view_of(self.active_player),
            terminated: self.terminated(),
            active_player: self.active_player,
            phase: self.phase,
            remaining_players,
            move_is_valid,
        }
    }

    fn all_rotated_tiles_of<'a>(
        &'a self,
        player: usize,
    ) -> Box<dyn Iterator<Item = Tile> + 'a> {
        Box::new(
            self.player_hands[player]
                .iter()
                .flat_map(|tile| (0..4).map(|rot| (*tile).rotated(rot))),
        )
    }

    fn move_is_allowed(&self, tile: Tile) -> bool {
        let tile_idx = self.active_player_hand_find_tile(tile);
        if tile_idx.is_none() {
            return false; // tile is not in the player's hand
        }
        let is_suicide = self.board.move_is_suicide(tile, self.active_player);
        if is_suicide {
            let mut all_possible_moves =
                self.all_rotated_tiles_of(self.active_player);
            if all_possible_moves
                .any(|tile| !self.board.move_is_suicide(tile, self.active_player))
            {
                return false;
            }
            // All moves are suicide
            return true;
        }
        true
    }

    fn active_player_hand_find_tile(&self, tile: Tile) -> Option<usize> {
        let mut hand = self.all_rotated_tiles_of(self.active_player);
        let idx = hand.position(|t| t == tile);
        idx.map(|idx| idx / 4)
    }

    fn eliminate_player(&mut self, player: usize) {
        self.board.eliminate_player(player);
        self.deck.append(&mut self.player_hands[player]);
        self.deck.shuffle(&mut rng());
        self.num_players_left -= 1;
    }
}
