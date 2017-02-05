extern crate rand;

use json::JsonValue;
use self::rand::{thread_rng, Rng};

pub struct State {
    initialized: bool,

    player_idx: u8,

    width: usize,
    height: usize,
    map: Vec<i8>,

    scores: Vec<u32>,
    tiles: Vec<u32>,
    alive: Vec<bool>
}

impl State {
    pub fn new() -> Self {
        State {
            initialized: false,

            player_idx: 0,

            width: 0,
            height: 0,
            map: vec![],

            scores: vec![],
            tiles: vec![],
            alive: vec![],
        }
    }

    pub fn handle_game_start(&mut self, data: &JsonValue) {
        self.player_idx = data["playerIndex"].as_u8().unwrap();
    }

    fn initialize(&mut self, data: &JsonValue) {
        self.initialized = true;
        self.width = data["map_diff"][2].as_number().unwrap().into();
        self.height = data["map_diff"][3].as_number().unwrap().into();
        self.map = vec![0; self.width * self.height * 2];
        for i in 4..data["map_diff"].len() {
            self.map[i - 4] = data["map_diff"][i].as_i8().unwrap();
        }

        self.scores = vec![0; data["scores"].len()];
        self.tiles = vec![0; data["scores"].len()];
        self.alive = vec![false; data["scores"].len()];
    }

    fn patch_map(&mut self, patch: &JsonValue) {
        let mut idx: isize = -2;
        let mut patch_idx: usize = 0;
        loop {
            idx += patch[patch_idx].as_isize().expect("Couldn't read skip patch count.");
            patch_idx = patch_idx + 1;
            if patch_idx >= patch.len() {
                break;
            }

            let patches = patch[patch_idx].as_usize().expect("Couldn't read apply patch count.");
            for i in 0..patches {
                self.map[idx as usize + i] = patch[patch_idx + 1 + i].as_i8().expect("Couldn't apply single patch.");
            }
            patch_idx += patches + 1;
            idx += patches as isize;

            if patch_idx >= patch.len() {
                break;
            }
        }
    }

    fn update_scores(&mut self, scores: &JsonValue) {
        for i in 0..scores.len() {
            self.scores[i] = scores[i]["total"].as_u32().unwrap();
            self.tiles[i] = scores[i]["tiles"].as_u32().unwrap();
            self.alive[i] = scores[i]["dead"].as_bool().unwrap();
        }
    }

    pub fn handle_game_update(&mut self, data: &JsonValue) {
        if !self.initialized {
            self.initialize(data);
            self.update_scores(&data["scores"]);
            return;
        }

        self.patch_map(&data["map_diff"]);
        self.update_scores(&data["scores"]);
    }

    fn xy_to_idx(&self, x: usize, y: usize) -> usize {
        return y * self.width + x;
    }

    fn own_valuable_square(&self, x: usize, y: usize) -> bool {
        self.map[y * self.width + x] > 1 &&
        self.map[self.width * self.height + y * self.width + x] == self.player_idx as i8
    }

    pub fn next_move(&self) -> Option<(usize, usize, bool)> {
        let mut squares: Vec<(usize, usize)> = vec![];
        for x in 0..self.width {
            for y in 0..self.height {
                if self.own_valuable_square(x, y) {
                    squares.push((x, y));
                }
            }
        }

        if squares.is_empty() {
            return None;
        }

        let square = squares[thread_rng().gen_range(0, squares.len())];

        const DX: [isize; 4] = [-1, 0, 1, 0];
        const DY: [isize; 4] = [0, -1, 0, 1];
        let dir: usize = thread_rng().gen_range(0, 4);
        for i in 0..4 {
            let nx = square.0 as isize + DX[(dir + i) % 4];
            let ny = square.1 as isize + DY[(dir + i) % 4];
            if nx < 0 || nx >= self.width as isize || ny < 0 || ny >= self.height as isize {
                continue;
            }

            return Some((self.xy_to_idx(square.0, square.1), self.xy_to_idx(nx as usize, ny as usize), false));
        }

        return None;
    }
}

