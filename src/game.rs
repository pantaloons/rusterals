use json::JsonValue;

#[derive(Debug)]
pub struct Game {
    pub initialized: bool,
    pub raw_map: Vec<i32>,

    pub idx: i32,
    pub turn: i32,

    pub width: usize,
    pub height: usize,

    pub scores: Vec<u32>,
    pub tiles: Vec<u32>,
    pub alive: Vec<bool>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            initialized: false,
            raw_map: vec![],

            idx: 0,
            turn: 0,

            width: 0,
            height: 0,

            scores: vec![],
            tiles: vec![],
            alive: vec![],
        }
    }

    pub fn handle_game_start(&mut self, data: &JsonValue) {
        self.idx = data["playerIndex"].as_i32().unwrap();
    }

    fn initialize(&mut self, data: &JsonValue) {
        self.initialized = true;
        self.width = data["map_diff"][2].as_number().unwrap().into();
        self.height = data["map_diff"][3].as_number().unwrap().into();
        self.raw_map = vec![0; self.width * self.height * 2];
        for i in 4..data["map_diff"].len() {
            self.raw_map[i - 4] = data["map_diff"][i].as_i32().unwrap();
        }

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
}
