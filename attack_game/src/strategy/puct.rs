use std::cell::Cell;

use crate::{
    evaluate,
    game::{retain_different_dest, retain_different_origin, AttackGame, Move, PlayerMove, Turn},
};

use super::{resolve_chance, StrategyExt};

use mcts::uct::a0puct as puct;
use rand::prelude::SliceRandom;
use risk_shared::map::{TerritoryId, EDGES};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Data {
    pub prediction: f64,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Params {
    pub c_puct: f64,
    pub c_puct_troops: f64,
    pub first_enemy_troop_reduction: f64,
    pub first_friendly_troop_reduction: f64,
    pub eval: evaluate::Params,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            c_puct: 0.6028558951476736,
            c_puct_troops: 0.702745173702057,
            first_enemy_troop_reduction: 0.13864357808607813,
            first_friendly_troop_reduction: 0.5888177056325355,
            eval: evaluate::Params::default(),
        }
    }
}

pub struct AttackPUCT<R: rand::Rng> {
    params: Params,
    root_game: AttackGame,
    game: AttackGame,
    rng: R,
    player_moves: Vec<PlayerMove>, // Cache this vec
    simulation_rounds: Cell<u64>,
}

impl<R: rand::Rng + Clone> StrategyExt for AttackPUCT<R> {
    type Rng = R;

    type Params = Params;

    fn from_rng(root_game: AttackGame, rng: R) -> Self {
        Self::from_params_rng(root_game, Params::default(), rng)
    }

    fn from_params_rng(root_game: AttackGame, params: Self::Params, rng: Self::Rng) -> Self {
        let game = root_game.clone();

        Self {
            params,
            root_game,
            game,
            rng,
            player_moves: Vec::new(),
            simulation_rounds: Cell::new(0),
        }
    }

    fn create_from(&self, root_game: AttackGame) -> Self {
        Self::from_params_rng(root_game, self.params, self.rng.clone())
    }

    fn root_game(&self) -> &AttackGame {
        &self.root_game
    }

    fn evaluate(&self, game: &AttackGame) -> f64 {
        evaluate::evaluate(game, &self.params.eval)
    }

    fn simulation_rounds(&self) -> u64 {
        self.simulation_rounds.get()
    }

    fn reset_simulation_rounds(&self) {
        self.simulation_rounds.set(0)
    }
}

impl<R: rand::Rng + Clone> mcts::Strategy for AttackPUCT<R> {
    type Action = Move;
    type Data = Data;

    fn reset(&mut self) {
        self.game = self.root_game.clone();
    }

    fn move_root(&mut self, action: &Self::Action) {
        self.root_game.make_move(*action);
        self.reset();
    }

    fn select(&mut self, node: &mcts::Node<Self::Action, Self::Data>) -> mcts::Selection {
        match self.game.turn() {
            Turn::Player | Turn::PlaceTroops => {
                let c_puct = if self.game.turn().is_player() {
                    self.params.c_puct
                } else {
                    self.params.c_puct_troops
                };

                if node.children.is_empty() {
                    assert_ne!(self.game.turn(), Turn::PlaceTroops);
                    mcts::Selection::Terminal(self.evaluate(&self.game))
                } else {
                    let selection = (0..node.children.len())
                        .fold((f64::NEG_INFINITY, 0), |(max, max_index), i| {
                            let puct = node.children[i].2.as_ref().map_or(f64::INFINITY, |x| {
                                puct(
                                    c_puct,
                                    x.score,
                                    x.visits,
                                    node.visits,
                                    node.children[i].1.prediction,
                                )
                            });

                            if puct > max {
                                (puct, i)
                            } else {
                                (max, max_index)
                            }
                        })
                        .1;

                    self.game.make_move(node.children[selection].0);
                    mcts::Selection::Selection(selection as u32)
                }
            }
            Turn::Chance(mov) => {
                let mov = resolve_chance(&self.game, mov, &mut self.rng);
                let mov_index = node
                    .children
                    .iter()
                    .position(|(x, _, _)| *x == mov)
                    .unwrap();
                self.game.make_move(mov);

                mcts::Selection::Selection(mov_index as u32)
            }
        }
    }

    fn expand(
        &mut self,
        _: &mcts::Node<Self::Action, Self::Data>,
    ) -> Vec<(Self::Action, Self::Data)> {
        match self.game.turn() {
            Turn::Player => {
                let mut actions = vec![];
                self.game.gen_player_moves_into(&mut actions);
                actions.shuffle(&mut self.rng);
                let mut actions = actions
                    .into_iter()
                    .map(|mov| {
                        let data = Data {
                            prediction: self.predictor(&self.game, mov).sqrt(),
                        };

                        (Move::Player(mov), data)
                    })
                    .collect::<Vec<_>>();

                let scale = actions
                    .iter()
                    .map(|(_, x)| x.prediction)
                    .sum::<f64>()
                    .recip();
                for (_, data) in &mut actions {
                    data.prediction *= scale;
                }

                actions
            }
            Turn::Chance(_) => {
                let mut actions = vec![];
                self.game.gen_chance_moves_into(&mut actions);
                actions
                    .into_iter()
                    .map(|(x, y)| (Move::Chance(x, y), Data { prediction: 0.0 }))
                    .collect()
            }
            Turn::PlaceTroops => {
                // Consider only placing on border territories
                let mut actions = TerritoryId::ALL
                    .into_iter()
                    .filter(|&x| {
                        self.game.occupier(x).is_p0()
                            && EDGES[x].iter().any(|&x| !self.game.occupier(x).is_p0())
                    })
                    .map(|x| {
                        let data = Data {
                            prediction: self.troops_predictor(&self.game, x).sqrt(),
                        };

                        (Move::PlaceTroops(x), data)
                    })
                    .collect::<Vec<_>>();

                let scale = actions
                    .iter()
                    .map(|(_, x)| x.prediction)
                    .sum::<f64>()
                    .recip();
                for (_, data) in &mut actions {
                    data.prediction *= scale;
                }

                actions
            }
        }
    }

    fn simulate(&mut self, _: &mcts::Node<Self::Action, Self::Data>) -> f64 {
        if self.game.turn().is_place_troops() {
            let moves = TerritoryId::ALL
                .into_iter()
                .filter(|&x| {
                    self.game.occupier(x).is_p0()
                        && EDGES[x].iter().any(|&x| !self.game.occupier(x).is_p0())
                })
                .collect::<Vec<_>>();

            while self.game.turn().is_place_troops() {
                self.game
                    .make_move(Move::PlaceTroops(*moves.choose(&mut self.rng).unwrap()))
            }
        }

        if let Turn::Chance(mov) = self.game.turn() {
            let mov = resolve_chance(&self.game, mov, &mut self.rng);
            self.game.make_move(mov);
        }

        let stand_pat = self.evaluate(&self.game);

        self.game.gen_player_moves_into(&mut self.player_moves);
        loop {
            match self.player_moves.len() {
                0 => break,
                1 => {
                    let player_move = self.player_moves[0];

                    if self.game.troops(player_move.origin) > 10
                        && self.game.troops(player_move.dest) > 10
                    {
                        let (attackers_lost, defenders_lost) = approx_resolve_chance(
                            self.game.troops(player_move.origin),
                            self.game.troops(player_move.dest),
                            &mut self.rng,
                        );

                        self.game.make_move(Move::Player(player_move));
                        self.game.make_chance_move(attackers_lost, defenders_lost);
                        self.simulation_rounds.set(self.simulation_rounds.get() + 1);
                    } else {
                        while self.game.troops(player_move.origin) > 1 {
                            self.game.make_move(Move::Player(player_move));

                            let chance_move =
                                resolve_chance(&self.game, player_move, &mut self.rng);
                            self.game.make_move(chance_move);
                            self.simulation_rounds.set(self.simulation_rounds.get() + 2);
                        }
                    }

                    self.game
                        .gen_player_moves_incremental(player_move, &mut self.player_moves);
                }
                _ => {
                    let player_move = fast_choose(&self.player_moves, &mut self.rng);

                    // There is no point in attacking a weak territory only once, so we will resolve it
                    if self.game.troops(player_move.origin) > 10
                        && self.game.troops(player_move.dest) > 10
                    {
                        let mut available_troops = [(0, TerritoryId::Alaska); 7];

                        available_troops[0] =
                            (self.game.troops(player_move.origin) - 1, player_move.origin);

                        // Add up the available troops in all adjacent territories
                        let mut territory_count = 1;
                        for ((troops, t), territory_id) in
                            available_troops
                                .iter_mut()
                                .zip(EDGES[player_move.dest].iter().filter(|&&t| {
                                    self.game.occupier(t).is_p0() && self.game.troops(t) > 1
                                }))
                        {
                            *t = *territory_id;
                            *troops = self.game.troops(*territory_id) - 1;
                            territory_count += 1;
                        }

                        available_troops[0..territory_count].sort_by_key(|(troops, _)| *troops);
                        let available_troops = &available_troops[0..territory_count];
                        let total_troops = 1 + available_troops
                            .iter()
                            .map(|(troops, _)| troops)
                            .sum::<u32>();

                        let (mut attackers_lost, defenders_lost) = approx_resolve_chance(
                            total_troops,
                            self.game.troops(player_move.dest),
                            &mut self.rng,
                        );

                        // Remove attackers, starting from the weakest territories
                        for (troops, territory) in available_troops {
                            if attackers_lost <= 1 {
                                break;
                            }

                            let troops_lost = std::cmp::min(attackers_lost - 1, *troops);
                            self.game.remove_troops(*territory, troops_lost);
                            attackers_lost -= troops_lost;
                        }

                        let (_, origin) = available_troops.last().unwrap();
                        let mov = Move::Player(PlayerMove {
                            origin: *origin,
                            dest: player_move.dest,
                        });

                        self.game.make_move(mov);
                        self.game.make_chance_move(1, defenders_lost);

                        // Incremental gen move
                        for (_, territory) in available_troops
                            .iter()
                            .filter(|(_, t)| self.game.troops(*t) == 1)
                        {
                            retain_different_origin(&mut self.player_moves, *territory)
                        }

                        if self.game.occupier(player_move.dest).is_p0()
                            && self.game.troops(player_move.dest) > 1
                        {
                            retain_different_dest(&mut self.player_moves, player_move.dest);

                            for &dest in EDGES[player_move.dest]
                                .iter()
                                .filter(|&&x| !self.game.occupier(x).is_p0())
                            {
                                self.player_moves.push(PlayerMove {
                                    origin: player_move.dest,
                                    dest,
                                });
                            }
                        }

                        self.simulation_rounds.set(self.simulation_rounds.get() + 3);
                    } else {
                        // Keep attacking the same target if we have more troops than they do
                        loop {
                            let chance_move =
                                resolve_chance(&self.game, player_move, &mut self.rng);
                            self.game.make_move(Move::Player(player_move));
                            self.game.make_move(chance_move);
                            self.simulation_rounds.set(self.simulation_rounds.get() + 1);

                            // If we only have one troop left, than than we cannot have more than
                            // the opponent, so we do not need to check that explicitly
                            if self.game.occupier(player_move.dest).is_p0()
                                || self.game.troops(player_move.origin)
                                    <= self.game.troops(player_move.dest)
                            {
                                break;
                            }
                        }

                        self.game
                            .gen_player_moves_incremental(player_move, &mut self.player_moves);
                    }
                }
            }
        }

        f64::max(stand_pat, self.evaluate(&self.game))
    }

    fn backpropagate(
        &mut self,
        score: f64,
        mut tree: mcts::TreeWalker<'_, '_, Self::Action, Self::Data>,
    ) {
        while let Some(node) = tree.pop() {
            node.visits += 1;
            node.score += score;
        }
    }
}

impl<R: rand::Rng> AttackPUCT<R> {
    fn predictor(&self, game: &AttackGame, mov: PlayerMove) -> f64 {
        self.territory_strength(game, mov.dest)
    }

    fn troops_predictor(&self, game: &AttackGame, territory: TerritoryId) -> f64 {
        self.territory_strength(game, territory).recip()
    }

    #[inline]
    fn territory_strength(&self, game: &AttackGame, territory: TerritoryId) -> f64 {
        let mut defenders = 0.0;
        let mut attackers = 0.0;
        if game.occupier(territory).is_p0() {
            attackers = game.troops(territory) as f64 - self.params.first_friendly_troop_reduction;
        } else {
            defenders = game.troops(territory) as f64 - self.params.first_enemy_troop_reduction;
        }

        for &territory in EDGES[territory] {
            let troops = game.troops(territory);
            if game.occupier(territory).is_p0() {
                attackers += troops as f64 - self.params.first_friendly_troop_reduction;
            } else {
                defenders += troops as f64 - self.params.first_enemy_troop_reduction;
            }
        }

        attackers / defenders
    }
}

fn fast_choose(moves: &[PlayerMove], rng: &mut impl rand::Rng) -> PlayerMove {
    let value = u64::from(rng.next_u32()) * moves.len() as u64;
    moves[(value >> 32) as usize]
}

fn approx_resolve_chance(
    attacking_troops: u32,
    defending_troops: u32,
    rng: &mut impl rand::Rng,
) -> (u32, u32) {
    const MEAN_ATTACKER_LOSS: f64 = 0.9641204;
    const MEAN_DEFENDER_LOSS: f64 = 1.03588;

    const RECIP_MEAN_ATTACKER_LOSS: f64 = 1.0 / MEAN_ATTACKER_LOSS;
    const RECIP_MEAN_DEFENDER_LOSS: f64 = 1.0 / MEAN_DEFENDER_LOSS;
    const SD: f64 = 0.8403248;

    let attackers = (attacking_troops - 1) as f64;
    let defenders = defending_troops as f64;
    let approx_rounds = f64::min(
        attackers * RECIP_MEAN_ATTACKER_LOSS.recip(),
        defenders * RECIP_MEAN_DEFENDER_LOSS.recip(),
    );
    let se = SD / approx_rounds.sqrt();

    let deviation = QUANTILES[(rng.next_u32() >> 24) as usize] * se;
    let attacker_loss = MEAN_ATTACKER_LOSS + deviation;
    let defender_loss = MEAN_DEFENDER_LOSS - deviation;

    let attacker_rounds = attackers / attacker_loss;
    let defender_rounds = defenders / defender_loss;
    if attacker_rounds > defender_rounds {
        let attackers_lost = std::cmp::min(
            (defender_rounds * attacker_loss).round() as u32,
            attacking_troops - 2,
        );
        let defenders_lost = defending_troops;
        (attackers_lost, defenders_lost)
    } else {
        let attackers_lost = attacking_troops - 1;
        let defenders_lost = std::cmp::min(
            (attacker_rounds * defender_loss).round() as u32,
            defending_troops - 1,
        );
        (attackers_lost, defenders_lost)
    }
}

const QUANTILES: [f64; 256] = include!("quantiles.txt");
