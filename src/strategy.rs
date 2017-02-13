use game::Game;
use time::precise_time_ns;
use rand::{weak_rng, Rng};
use std::collections::BTreeMap;

const WIN_VALUE: i32 = 10_000;
const LOSS_VALUE: i32 = -10_000;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct Pair {
    pub x: usize,
    pub y: usize,
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

#[derive(Clone)]
struct Tile {
    kind: TileType,
    owner: i32,
    count: i32,
}

#[derive(Clone)]
struct State {
    width: usize,
    height: usize,

    player_idx: i32,

    tiles: Vec<Vec<Tile>>,
    player_turn: usize,
    global_turn: usize,

    scores: Vec<u32>,
    land: Vec<u32>,
    cities: Vec<u32>,
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
struct Action {
    src: Pair,
    dst: Pair,
}

struct ActionTransfer {
    visit_count: i32,
    cumulative_score: i32,

    next_node: SearchNode,
}

impl ActionTransfer {
    pub fn update_reward(&mut self, score: i32) {
        self.visit_count += 1;
        self.cumulative_score += score;
    }
}

struct SearchNode {
    state: State,
    actions: BTreeMap<Option<Action>, ActionTransfer>,
}

impl SearchNode {
    pub fn new(state: State) -> Self {
        SearchNode {
            state: state,
            actions: BTreeMap::new(),
        }
    }

    pub fn search(&mut self, depth: i32) -> i32 {
        if self.state.is_game_over() {
            return if self.state.is_player_winner() {
                WIN_VALUE
            } else {
                LOSS_VALUE
            };
        } else if depth == 10 {
            return self.state.evaluate();
        }

        let action = self.select_action();
        let mut action_transfer: &mut ActionTransfer = self.get_action_transfer(&action);
        let score: f32 = 0.9 * action_transfer.next_node.search(depth + 1) as f32;
        action_transfer.update_reward(score as i32);
        score as i32
    }

    pub fn select_action(&self) -> Option<Action> {
        self.state.select_action()
    }

    pub fn get_action_transfer(&mut self, action: &Option<Action>) -> &mut ActionTransfer {
        if !self.actions.contains_key(action) {
            self.actions.insert(action.to_owned(),
                                ActionTransfer {
                                    visit_count: 0,
                                    cumulative_score: 0,

                                    next_node: SearchNode::new(self.state.simulate_action(action)),
                                });
        }
        self.actions.get_mut(action).unwrap()
    }

    pub fn select_best_move(&self) -> Option<Action> {
        let mut best_score: i32 = -1;
        let mut best_action: Option<Option<Action>> = None;
        for (action, transfer) in &self.actions {
            if transfer.visit_count > best_score {
                best_score = transfer.visit_count;
                best_action = Some(action.to_owned());
            }
        }

        best_action.unwrap_or_default()
    }
}

impl State {
    pub fn new(game: &Game) -> Self {
        let mut state: State = State {
            width: game.width,
            height: game.height,

            player_idx: game.idx,

            tiles: vec![],
            player_turn: 0,
            global_turn: 0,

            scores: vec![],
            land: vec![],
            cities: vec![],
        };

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

                state.tiles[i].push(Tile {
                    kind: kind,
                    owner: value,
                    count: game.raw_map[i * game.width + j],
                });
            }
        }

        state.scores = game.scores.clone();
        state.land = game.tiles.clone();
        state.cities = vec![0; game.scores.len()];

        state
    }

    pub fn is_game_over(&self) -> bool {
        let mut found_player_with_squares: bool = false;
        for score in &self.scores {
            if score > &0 {
                if found_player_with_squares {
                    return false;
                }

                found_player_with_squares = true;
            }
        }

        true
    }

    pub fn is_player_winner(&self) -> bool {
        assert!(self.is_game_over());
        self.scores[self.player_idx as usize] > 0
    }

    pub fn select_action(&self) -> Option<Action> {
        let mut moves: Vec<Action> = vec![];
        for i in 0..self.height {
            for j in 0..self.width {
                if self.tiles[i][j].owner == self.player_turn as i32 && self.tiles[i][j].count > 1 {
                    const DX: [isize; 4] = [-1, 0, 1, 0];
                    const DY: [isize; 4] = [0, -1, 0, 1];
                    for k in 0..4 {
                        let nx = j as isize + DX[k];
                        let ny = i as isize + DY[k];
                        if nx >= 0 && ny < self.width as isize && ny >= 0 &&
                           ny < self.height as isize {
                            let nx: usize = nx as usize;
                            let ny: usize = ny as usize;
                            if self.tiles[ny][nx].kind == TileType::Mountain {
                                continue;
                            }

                            moves.push(Action {
                                src: Pair { x: j, y: i },
                                dst: Pair { x: nx, y: ny },
                            });
                        }
                    }
                }
            }
        }

        let index = weak_rng().gen_range(0, moves.len() + 1);
        if index == moves.len() {
            // We always have the option to do nothing.
            None
        } else {
            Some(moves[index].clone())
        }
    }

    pub fn simulate_action(&self, action: &Option<Action>) -> Self {
        let mut next = self.clone();

        if action.is_some() {
            let action: &Action = action.as_ref().unwrap();
            let src_count: i32 = next.tiles[action.src.y][action.src.x].count;
            let src_owner: usize = next.tiles[action.src.y][action.src.x].owner as usize;

            let mut dst = &mut next.tiles[action.dst.y][action.dst.x];
            let dst_owner: usize = dst.owner as usize;

            if dst.owner >= 0 {
                if dst_owner == src_owner {
                    dst.count += src_count - 1;
                } else if src_count - 1 > dst.count {
                    if dst.kind == TileType::Plain {
                        next.scores[src_owner] -= (dst.count - (src_count - 1)) as u32;
                        next.scores[dst_owner] -= dst.count as u32;
                        next.land[dst_owner] -= 1;
                        next.land[src_owner] += 1;
                        dst.count = src_count - 1 - dst.count;
                        dst.owner = src_owner as i32;
                    } else if dst.kind == TileType::City {
                        next.scores[src_owner] -= (dst.count - (src_count - 1)) as u32;
                        next.scores[dst_owner] -= dst.count as u32;
                        next.land[dst_owner] -= 1;
                        next.land[src_owner] += 1;
                        dst.count = src_count - 1 - dst.count;
                        dst.owner = src_owner as i32;
                        next.cities[dst_owner] -= 1;
                        next.cities[src_owner] += 1;
                    } else if dst.kind == TileType::General {
                        dst.kind = TileType::City;

                        // Transfer score first from general battle.
                        next.scores[src_owner] -= (dst.count - (src_count - 1)) as u32;
                        next.scores[dst_owner] -= dst.count as u32;

                        // Now transfer score to winner.
                        next.scores[src_owner] += next.scores[dst_owner];
                        next.scores[dst_owner] = 0;

                        next.land[src_owner] += next.land[dst_owner];
                        next.land[dst_owner] = 0;

                        next.cities[src_owner] += next.cities[dst_owner];
                        next.cities[dst_owner] = 0;

                        // Correct model here is to generate with random probabilities
                        // what squares this player might have owned, and transfer them
                        // as part of the state transfer weight. This isn't feasible,
                        // instead just always terminate search here as being good.
                    } else {
                        panic!("Bad tile found.");
                    }
                } else {
                    dst.count = dst.count - (src_count - 1);
                }
            } else if dst.kind == TileType::City {
                if src_count - 1 > dst.count {
                    next.cities[src_owner] += 1;
                    next.land[src_owner] += 1;
                    dst.owner = src_owner as i32;
                } else {
                    // TODO: City will start regenerating units. Implement that.
                    dst.count -= src_count - 1;
                }
            } else {
                assert_eq!(dst.kind, TileType::Plain);
                assert!(src_count > 1);
                dst.count = src_count - 1;
                next.land[src_owner] += 1;
            }
        }
        if action.is_some() {
            let action: &Action = action.as_ref().unwrap();
            next.tiles[action.src.y][action.src.x].count = 1;
        }

        if next.player_turn == next.scores.len() - 1 {
            next.global_turn += 1;
        }
        next.player_turn = (next.player_turn + 1) % (next.scores.len());

        if next.global_turn % 50 == 0 {
            for i in 0..next.height {
                for j in 0..next.width {
                    if next.tiles[i][j].owner >= 0 {
                        next.tiles[i][j].count += next.cities[next.tiles[i][j].owner as usize] as
                                                  i32;
                    }
                }
            }
        }

        for i in 0..next.scores.len() {
            next.scores[i] += next.cities[i] * next.land[i];
        }

        next
    }

    pub fn evaluate(&self) -> i32 {
        let mut score: i32 = 0;

        for i in 0..self.width {
            for j in 0..self.height {
                if self.tiles[j][i].owner == self.player_idx {
                    score += 9;
                    score += (self.tiles[j][i].count as f32).powf(1.5) as i32;
                }
            }
        }

        score
    }
}

pub struct MonteCarlo {
    root: Option<SearchNode>,
}

impl MonteCarlo {
    pub fn new() -> Self {
        MonteCarlo { root: None }
    }

    pub fn initialize(&mut self, game: &Game) {
        self.root = Some(SearchNode::new(State::new(game)));
    }

    pub fn next_move(&mut self) -> Option<(Pair, Pair, bool)> {
        assert!(self.root.is_some());
        let start = precise_time_ns();

        let mut count = 0;
        loop {
            count += 1;
            self.root.as_mut().unwrap().search(0);
            if precise_time_ns() - start > 450_000_000 {
                break;
            }
        }

        println!("Sampled {:?} walks for next move.", &count);

        let next_move = self.root.as_ref().unwrap().select_best_move();
        match next_move {
            None => None,
            Some(action) => Some((action.src, action.dst, false)),
        }
    }
}
