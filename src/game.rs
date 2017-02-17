use json::JsonValue;

#[derive(Debug)]
pub struct Game {
    pub initialized: bool,
    pub raw_map: Vec<i32>,

    pub player_index: usize,
    pub turn: i32,

    pub width: usize,
    pub height: usize,

    pub generals: Vec<isize>,
    pub cities: Vec<usize>,
    pub scores: Vec<u32>,
    pub tiles: Vec<u32>,
    pub alive: Vec<bool>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            initialized: false,
            raw_map: vec![],

            player_index: 0,
            turn: 0,

            width: 0,
            height: 0,

            generals: vec![],
            cities: vec![],
            scores: vec![],
            tiles: vec![],
            alive: vec![],
        }
    }

    pub fn handle_game_start(&mut self, data: &JsonValue) {
        self.player_index = data["playerIndex"].as_usize().unwrap();
    }

    fn initialize(&mut self, data: &JsonValue) {
        self.initialized = true;
        self.width = data["map_diff"][2].as_number().unwrap().into();
        self.height = data["map_diff"][3].as_number().unwrap().into();
        self.raw_map = vec![0; self.width * self.height * 2];
        for i in 4..data["map_diff"].len() {
            self.raw_map[i - 4] = data["map_diff"][i].as_i32().unwrap();
        }

        self.generals = vec![0; data["scores"].len()];
        for i in 0..data["generals"].len() {
            self.generals[i] = data["generals"][i].as_isize().unwrap().into();
        }
        self.patch_cities(&data["cities_diff"]);
        self.scores = vec![0; data["scores"].len()];
        self.tiles = vec![0; data["scores"].len()];
        self.alive = vec![false; data["scores"].len()];
    }

    fn patch_map(&mut self, patch: &JsonValue) {
        let mut idx: isize = -2;
        let mut patch_idx: usize = 0;
        loop {
            idx += patch[patch_idx].as_isize().unwrap();
            patch_idx += 1;
            if patch_idx >= patch.len() {
                break;
            }

            let patches = patch[patch_idx].as_usize().unwrap();
            for i in 0..patches {
                self.raw_map[idx as usize + i] = patch[patch_idx + 1 + i].as_i32().unwrap();
            }
            patch_idx += patches + 1;
            idx += patches as isize;

            if patch_idx >= patch.len() {
                break;
            }
        }
    }

    fn patch_cities(&mut self, patch: &JsonValue) {
        let mut new_cities: Vec<usize> = Vec::new();
        let mut idx: usize = 0;
        let mut patch_idx: usize = 0;
        loop {
            let matching = patch[patch_idx].as_usize().unwrap();
            for i in 0..matching {
                new_cities.push(self.cities[idx + i])
            }
            patch_idx += 1;
            if patch_idx >= patch.len() {
                break;
            }
            idx += matching;

            let mismatching = patch[patch_idx].as_usize().unwrap();
            for i in 0..mismatching {
                new_cities.push(patch[patch_idx + 1 + i].as_usize().unwrap());
            }
            patch_idx += mismatching + 1;
            idx += mismatching;

            if patch_idx >= patch.len() {
                break;
            }
        }

        self.cities = new_cities;
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

        self.turn = data["turn"].as_i32().unwrap();
        self.patch_map(&data["map_diff"]);
        self.patch_cities(&data["cities_diff"]);
        for i in 0..data["generals"].len() {
            self.generals[i] = data["generals"][i].as_isize().unwrap();
        }
        self.update_scores(&data["scores"]);
    }
}
