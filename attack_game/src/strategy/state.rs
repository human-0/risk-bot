use enum_map::EnumMap;
use risk_shared::{map::TerritoryId, player::PlayerId};

use super::StrategyExt;
use crate::game::{AttackGame, Move, PlayerMove};

pub trait StatefulStrategy {
    type Rng: rand::RngCore;
    type Params;

    fn from_rng(rng: Self::Rng) -> Self;
    fn with_params(params: Self::Params, rng: Self::Rng) -> Self;

    fn reset_new(&mut self);

    fn reset(
        &mut self,
        troops: EnumMap<TerritoryId, u32>,
        occupiers: EnumMap<TerritoryId, PlayerId>,
        card_sets_redeemed: u32,
    );

    fn place_troops(
        &mut self,
        troop_count: u32,
        troops: EnumMap<TerritoryId, u32>,
        occupiers: EnumMap<TerritoryId, PlayerId>,
        distributions: &mut EnumMap<TerritoryId, u32>,
        card_sets_redeemed: u32,
    );

    fn get_move(&mut self) -> Option<(PlayerMove, bool)>;

    fn make_moves(&mut self, player: PlayerMove, chance: (u8, u8));
}

#[repr(transparent)]
pub struct State<'a, S: StrategyExt> {
    mcts: mcts::MCTS<'a, S>,
}

impl<'a, S: StrategyExt> StatefulStrategy for State<'a, S> {
    type Rng = S::Rng;
    type Params = S::Params;

    fn from_rng(rng: S::Rng) -> Self {
        Self {
            mcts: mcts::MCTS::new(S::from_rng(AttackGame::new(), rng)),
        }
    }

    fn with_params(params: S::Params, rng: S::Rng) -> Self {
        Self {
            mcts: mcts::MCTS::new(S::from_params_rng(AttackGame::new(), params, rng)),
        }
    }

    fn reset_new(&mut self) {
        let game = AttackGame::new();
        self.mcts = mcts::MCTS::new(self.mcts.strategy().create_from(game));
    }

    fn reset(
        &mut self,
        troops: EnumMap<TerritoryId, u32>,
        occupiers: EnumMap<TerritoryId, PlayerId>,
        card_sets_redeemed: u32,
    ) {
        let mut game = AttackGame::new();
        game.set_state(troops, occupiers, card_sets_redeemed);

        self.mcts = mcts::MCTS::new(self.mcts.strategy().create_from(game));
    }

    fn place_troops(
        &mut self,
        mut troop_count: u32,
        troops: EnumMap<TerritoryId, u32>,
        occupiers: EnumMap<TerritoryId, PlayerId>,
        distributions: &mut EnumMap<TerritoryId, u32>,
        card_sets_redeemed: u32,
    ) {
        let mut game = AttackGame::new();
        game.set_state(troops, occupiers, card_sets_redeemed);
        game.set_troops_to_place(troop_count);

        self.mcts = mcts::MCTS::new(self.mcts.strategy().create_from(game));

        while self.mcts.strategy().root_game().turn().is_place_troops() {
            let nodes = self.calculate_nodes();
            let nodes = (nodes as f64 * f64::max(1.0, (troop_count as f64).ln())) as u32;
            let Move::PlaceTroops(territory) = self.calculate_move(nodes) else {
                unreachable!();
            };

            self.mcts.move_root(Move::PlaceTroops(territory));

            let troops_placed = troop_count - self.mcts.strategy().root_game().troops_to_place();
            troop_count -= troops_placed;
            distributions[territory] += troops_placed;
        }
    }

    fn get_move(&mut self) -> Option<(PlayerMove, bool)> {
        if self.mcts.root().children.is_empty() {
            return None;
        }

        let nodes = self.calculate_nodes();
        let mov = self.calculate_move(nodes);
        let eval = self
            .mcts
            .strategy()
            .evaluate(self.mcts.strategy().root_game());

        if self
            .mcts
            .root()
            .children
            .iter()
            .filter_map(|x| x.2.as_ref())
            .all(|x| (x.score / x.visits as f64) < eval)
        {
            return None;
        }

        let Move::Player(mov) = mov else {
            unreachable!()
        };

        assert!(self.mcts.strategy().root_game().troops(mov.origin) > 1);
        assert_eq!(
            self.mcts.strategy().root_game().occupier(mov.origin),
            PlayerId::P0
        );
        assert_ne!(
            self.mcts.strategy().root_game().occupier(mov.dest),
            PlayerId::P0
        );

        let repeat = self.mcts.root().children.len() == 1;
        Some((mov, repeat))
    }

    fn make_moves(&mut self, player: PlayerMove, chance: (u8, u8)) {
        self.mcts.move_root(Move::Player(player));
        self.mcts.move_root(Move::Chance(chance.0, chance.1));
    }
}

impl<'a, S: StrategyExt> State<'a, S> {
    fn calculate_nodes(&self) -> u32 {
        let game = self.mcts.strategy().root_game();
        std::cmp::min(400, game.players_remaining() as u32 * 100)
            .min(50 * game.territories_occupied() as u32)
            .min(25 * (TerritoryId::ALL.len() - game.territories_occupied()) as u32)
    }

    fn calculate_move(&mut self, nodes: u32) -> Move {
        self.mcts.strategy().reset_simulation_rounds();
        let mut next_check = 0;
        for i in 0..nodes {
            if i >= next_check {
                if let Some((most_visits, second_most_visits)) = self.top_two_visits() {
                    if most_visits - second_most_visits > nodes - i {
                        break;
                    }

                    next_check += most_visits - second_most_visits + 1;
                }
            }

            if i > 20 && self.mcts.strategy().simulation_rounds() >= 10000 {
                break;
            }

            self.mcts.add_node();
        }

        *self.mcts.most_visits().unwrap().0
    }

    fn top_two_visits(&self) -> Option<(u32, u32)> {
        if self.mcts.root().children.len() <= 1 {
            return None;
        }

        let mut most_visits = 0;
        let mut second_most_visits = 0;
        for child in self
            .mcts
            .root()
            .children
            .iter()
            .filter_map(|(_, _, node)| node.as_ref())
        {
            if child.visits > second_most_visits {
                second_most_visits = child.visits;
                if second_most_visits > most_visits {
                    std::mem::swap(&mut most_visits, &mut second_most_visits);
                }
            }
        }

        Some((most_visits, second_most_visits))
    }

    pub fn mcts(&self) -> &mcts::MCTS<'a, S> {
        &self.mcts
    }
}
