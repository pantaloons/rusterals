use game::Game;
use state::{State, Action, Pair};
use time::precise_time_ns;
use std::collections::BTreeMap;
use std::f32;
use rand::{weak_rng, Rng, XorShiftRng};
use std::fmt;

#[derive(Debug)]
struct ActionTransfer {
    visit_count: usize,
    cumulative_score: i32,

    next_node: SearchNode,
}

impl Default for ActionTransfer {
    fn default() -> ActionTransfer {
        ActionTransfer {
            visit_count: 0,
            cumulative_score: 0,
            next_node: SearchNode::new(),
        }
    }
}

#[derive(Debug)]
struct SearchNode {
    actions: BTreeMap<Action, ActionTransfer>,
}

impl SearchNode {
    pub fn new() -> Self {
        SearchNode { actions: BTreeMap::new() }
    }

    pub fn search<T: Rng>(&mut self, rng: &mut T, state: &mut State, depth: i32) -> i32 {
        if depth == 0 {
            return state.evaluate();
        }

        let action = self.select_action(rng, state);
        let counts_before = state.apply_action(action);
        let transfer: &mut ActionTransfer = self.get_action_transfer(action);
        let score = transfer.next_node.search(rng, state, depth - 1);
        state.unapply_action(action, counts_before);
        transfer.visit_count += 1;
        transfer.cumulative_score += score;
        score as i32
    }

    pub fn select_action<T: Rng>(&self, rng: &mut T, state: &mut State) -> Action {
        state.select_action(rng)
    }

    pub fn get_action_transfer(&mut self, action: Action) -> &mut ActionTransfer {
        self.actions.entry(action).or_insert_with(ActionTransfer::default)
    }

    pub fn select_best_move(&self) -> Action {
        let mut best_score: f32 = f32::MIN;
        let mut best_action: Option<Action> = None;
        for (action, transfer) in &self.actions {
            let next_score = transfer.cumulative_score as f32 / transfer.visit_count as f32;
            if next_score > best_score {
                best_score = next_score;
                best_action = Some(action);
            }
        }

        best_action.unwrap()
    }
}

pub struct MonteCarlo {
    root: Option<SearchNode>,
    rng: XorShiftRng,
}

impl fmt::Debug for MonteCarlo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.root.fmt(f)
    }
}

impl MonteCarlo {
    pub fn new() -> Self {
        MonteCarlo {
            root: None,
            rng: weak_rng(),
        }
    }

    pub fn next_move(&mut self, game: &Game) -> Action {
        let start = precise_time_ns();

        self.root = Some(SearchNode::new());
        let mut state = State::new(game);

        let mut count = 0;
        let depth = 50 - (game.turn % 50);
        loop {
            count += 1;
            self.root.as_mut().unwrap().search(&mut self.rng, &mut state, depth);
            if precise_time_ns() - start > 200_000_00000 {
                break;
            }
        }

        println!("Sampled {:?} walks for next move. Depth as {:#?} {:#?} {:#?}.",
                 &count,
                 depth,
                 start,
                 precise_time_ns());
        self.root.as_ref().unwrap().select_best_move();
    }
}

#[cfg(test)]
mod tests {
    use strategy::MonteCarlo;
    use game::Game;
    use state::State;
    use state::Action;
    use state::Pair;

    #[test]
    fn test_large() {
        let game: Game = Game {
            initialized: true,
            raw_map: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                          3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -3, -3, -3,
                          -3, -1, -1, -1, -3, -3, -3, -3, -3, -3, -4, -3, -3, -3, -3, -4, -3, -3,
                          -3, -3, -1, 0, -1, -3, -4, -4, -4, -4, -4, -3, -3, -4, -3, -3, -3, -4,
                          -4, -3, -3, -2, -1, -1, -3, -3, -3, -4, -3, -3, -3, -3, -3, -3, -3, -3,
                          -3, -3, -4, -3, -4, -3, -3, -3, -3, -3, -3, -3, -4, -3, -3, -4, -3, -4,
                          -4, -3, -3, -3, -3, -4, -3, -3, -3, -3, -3, -3, -3, -4, -3, -3, -4, -3,
                          -4, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -4, -3, -3, -3, -3, -3,
                          -3, -3, -4, -3, -4, -3, -3, -3, -3, -3, -4, -3, -4, -3, -3, -3, -3, -3,
                          -3, -4, -3, -3, -4, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -4, -4,
                          -3, -3, -3, -3, -3, -3, -3, -3, -3, -4, -3, -4, -3, -3, -3, -3, -3, -4,
                          -3, -3, -3, -3, -3, -3, -3, -3, -4, -3, -3, -4, -3, -3, -3, -4, -4, -3,
                          -3, -3, -3, -3, -3, -3, -4, -3, -3, -3, -3, -3, -4, -3, -4, -3, -3, -3,
                          -4, -4, -3, -3, -3, -3, -3, -3, -4, -3, -3, -4, -3, -3, -4, -3, -4, -3,
                          -3, -3, -4, -3, -3, -3, -3, -4, -3, -3, -3, -3, -3, -3, -4, -3, -3, -3,
                          -3, -4, -4, -3, -4, -4, -3, -3, -3, -3, -4, -3, -3, -4, -3, -3, -3, -3,
                          -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3,
                          -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -4, -3, -3, -4, -3, -3, -3, -3,
                          -4, -3, -4, -3, -3, -3, -3, -3, -3, -3, -3, -4, -4, -3, -3, -3, -3, -3,
                          -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -3, -4, -4, -3, -3, -4, -4, -3,
                          -3, -3, -3, -3, -3, -3, -3, -3, -4, -3, -3, -3, -3, -3, -3, -4, -4, -3,
                          -4, -3, -3, -3, -4, -3, -3, -3, -3, -4, -3, -3, -3, -3, -3, -3, -4, -3,
                          -3, -3, -3, -3, -3, -3, -3, -4, -4, -4, -3, -4, -4, -3, -4, -3, -4],
            player_index: 0,
            turn: 5,
            width: 19,
            height: 20,
            generals: vec![24, -1],
            cities: vec![],
            scores: vec![3, 3],
            tiles: vec![1, 1],
            alive: vec![false, false],
        };

        let mut search: MonteCarlo = MonteCarlo::new();
        let next_move = search.next_move(&game);
    }

    #[test]
    fn test_search() {
        let mut search: MonteCarlo = MonteCarlo::new();
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

        let next_move = search.next_move(&game);
        println!("Next move: {:?}", next_move);
    }
}