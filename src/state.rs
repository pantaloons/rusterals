use game::Game;
use rand::Rng;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct Pair {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct Action {
    pub half: bool,
    pub src: Pair,
    pub dst: Pair,
}

#[derive(Clone, PartialEq, Debug)]
enum TileType {
    Plain,
    City,
    Mountain,
    Fog,
    Obstacle,
    General,
}

#[derive(Clone, Debug)]
struct Tile {
    kind: TileType,
    owner: usize,
    count: u32,
}

#[derive(Clone, Debug)]
pub struct State {
    width: usize,
    height: usize,
    player_count: usize,

    tiles: Vec<Vec<Tile>>,
    global_turn: usize,

    scores: Vec<u32>,
    land: Vec<u32>,
    cities: Vec<u32>,

    pub owned_tiles: Vec<Vec<Pair>>,
    search_scratch: Vec<Action>,
}

impl State {
    pub fn new(game: &Game) -> Self {
        let mut state: State = State {
            width: game.width,
            height: game.height,
            player_count: game.alive.len(),

            tiles: vec![],
            global_turn: game.turn as usize,

            scores: vec![],
            land: vec![],
            cities: vec![],

            owned_tiles: vec![],
            search_scratch: Vec::with_capacity(500),
        };

        let num_players = game.alive.len();
        let player_shift = game.alive.len() - game.player_index;

        for i in 0..game.height {
            state.tiles.push(vec![]);
            for j in 0..game.width {
                let value = game.raw_map[game.width * game.height + i * game.width + j];

                let kind: TileType = match value {
                    -2 => TileType::Mountain,
                    -3 => TileType::Fog,
                    -4 => TileType::Obstacle,
                    -1 | _ => TileType::Plain,
                };

                let owner: usize = if value < 0 {
                    num_players
                } else {
                    (value as usize + player_shift) % num_players
                };
                state.tiles[i].push(Tile {
                    kind: kind,
                    owner: owner,
                    count: game.raw_map[i * game.width + j] as u32,
                });
            }
        }

        for general in &game.generals {
            if *general == -1 {
                continue;
            }

            state.tiles[*general as usize / game.width][*general as usize % game.width].kind = TileType::General;
        }

        for city in &game.cities {
            state.tiles[city / game.width][city % game.width].kind = TileType::City;
        }

        state.scores = game.scores.clone();
        state.land = game.tiles.clone();
        state.cities = vec![0; game.scores.len()];

        for _ in 0..state.player_count {
            state.owned_tiles.push(Vec::with_capacity(500));
        }
        for i in 0..state.height {
            for j in 0..state.width {
                if state.tiles[i][j].owner == 0 {
                    state.owned_tiles[0].push(Pair { x: j, y: i });
                    if state.tiles[i][j].kind == TileType::City || state.tiles[i][j].kind == TileType::General {
                        state.cities[0] += 1;
                    }
                }
            }
        }

        state
    }

    pub fn select_action<T: Rng>(&mut self, rng: &mut T) -> Option<Action> {
        const DX: [isize; 4] = [-1, 0, 1, 0];
        const DY: [isize; 4] = [0, -1, 0, 1];

        self.search_scratch.clear();
        for ref tile in &self.owned_tiles[0] {
            if self.tiles[tile.y][tile.x].count <= 1 {
                continue;
            }

            for k in 0..4 {
                let nx = tile.x as isize + DX[k];
                let ny = tile.y as isize + DY[k];
                if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                    let nx: usize = nx as usize;
                    let ny: usize = ny as usize;
                    if self.tiles[ny][nx].kind == TileType::Mountain ||
                       self.tiles[ny][nx].kind == TileType::City ||
                       self.tiles[ny][nx].kind == TileType::Obstacle {
                        continue;
                    }

                    self.search_scratch.push(Action {
                        half: false,
                        src: Pair { x: tile.x, y: tile.y },
                        dst: Pair { x: nx, y: ny },
                    });

                    if self.tiles[tile.y][tile.x].count > 2 {
                        self.search_scratch.push(Action {
                            half: true,
                            src: Pair { x: tile.x, y: tile.y },
                            dst: Pair { x: nx, y: ny },
                        });
                    }
                }
            }
        }

        let index = rng.gen_range(0, self.search_scratch.len() + 1);
        if index == self.search_scratch.len() {
            // We always have the option to do nothing.
            None
        } else {
            Some(self.search_scratch.swap_remove(index))
        }
    }

    pub fn unapply_action(&mut self, action: &Option<Action>, counts: (u32, u32)) {
        if self.global_turn % 50 == 0 {
            for ref tile in &self.owned_tiles[0] {
                self.tiles[tile.y][tile.x].count -= self.cities[0];
            }
            self.scores[0] -= self.owned_tiles[0].len() as u32 * self.cities[0];
        }
        else if self.global_turn % 2 == 0 {
            let mut count: u32 = 0;
            for ref tile in &self.owned_tiles[0] {
                if self.tiles[tile.y][tile.x].kind == TileType::City ||
                   self.tiles[tile.y][tile.x].kind == TileType::General {
                    count += 1;
                    self.tiles[tile.y][tile.x].count -= self.cities[0];
                }
            }
            self.scores[0] -= count * self.cities[0];
        }

        self.global_turn -= 1;

        if action.is_some() {
            let action: &Action = action.as_ref().unwrap();
            if counts.1 == 0 {
                self.owned_tiles[0].pop();
                self.land[0] -= 1;
                self.tiles[action.dst.y][action.dst.x].owner = self.player_count;
            }
            self.tiles[action.src.y][action.src.x].count = counts.0;
            self.tiles[action.dst.y][action.dst.x].count = counts.1;
        }
    }

    pub fn apply_action(&mut self, action: &Option<Action>) -> (u32, u32) {
        let mut src_before: u32 = 0;
        let mut dst_before: u32 = 0;

        if action.is_some() {
            let action: &Action = action.as_ref().unwrap();

            src_before = self.tiles[action.src.y][action.src.x].count;
            dst_before = self.tiles[action.dst.y][action.dst.x].count;

            if action.half {
                self.tiles[action.dst.y][action.dst.x].count += self.tiles[action.src.y][action.src.x].count / 2;
                self.tiles[action.src.y][action.src.x].count = (self.tiles[action.src.y][action.src.x].count + 1) / 2;
            }
            else {
                self.tiles[action.dst.y][action.dst.x].count += self.tiles[action.src.y][action.src.x].count - 1;
                self.tiles[action.src.y][action.src.x].count = 1;
            }

            if dst_before == 0 {
                self.land[0] += 1;
                self.owned_tiles[0].push(action.dst.clone());
                self.tiles[action.dst.y][action.dst.x].owner = 0;
            }
        }

        self.global_turn += 1;
        if self.global_turn % 50 == 0 {
            for ref tile in &self.owned_tiles[0] {
                self.tiles[tile.y][tile.x].count += self.cities[0];
            }
            self.scores[0] += self.owned_tiles[0].len() as u32 * self.cities[0];
        }
        else if self.global_turn % 2 == 0 {
            let mut count: u32 = 0;
            for ref tile in &self.owned_tiles[0] {
                if self.tiles[tile.y][tile.x].kind == TileType::City ||
                   self.tiles[tile.y][tile.x].kind == TileType::General {
                    count += 1;
                    self.tiles[tile.y][tile.x].count += self.cities[0];
                }
            }
            self.scores[0] += count * self.cities[0];
        }

        (src_before, dst_before)
    }

    #[inline]
    pub fn evaluate(&self) -> i32 {
        self.land[0] as i32
    }
}

#[cfg(test)]
mod tests {
    use game::Game;
    use state::State;
    use state::Action;
    use state::Pair;
    use rand::weak_rng;

    #[test]
    fn test_apply_action1() {
        let game: Game = Game {
            initialized: true,
            player_index: 0,
            turn: 0,

            width: 3,
            height: 3,

            cities: vec![3],
            generals: vec![3, -1],
            scores: vec![2, 2],
            tiles: vec![1, 1],
            alive: vec![true, true],

            raw_map: vec![0, 0, 0, 2, 0, 0, 0, 0, 0, -1, -1, -3, 0, -1, -3, -1, -1, -3],
        };

        let mut state = State::new(&game);
        state.apply_action(&Some(Action { half : false, src : Pair { x : 0, y : 1 }, dst : Pair { x : 0, y : 0 }}));
        println!("State: {:#?}", state);

        for _ in 0..10 {
            println!("Action: {:#?}", state.select_action(&mut weak_rng()));
        }
    }
}