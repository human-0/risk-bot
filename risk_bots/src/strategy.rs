use std::collections::VecDeque;

use attack_game::{
    game::PlayerMove,
    strategy::{
        puct,
        state::{State, StatefulStrategy},
    },
};
use enum_map::EnumMap;
use risk_helper::{state::ClientState, util, ManagedPlayer};
use risk_shared::{
    map::{TerritoryId, EDGES},
    player::PlayerId,
    query::QueryDetails,
    record::{
        Cause, Move, MoveAttack, MoveDefend, MoveDistributeTroops, MoveFortify, MoveRedeemCards,
        MoveTroopsAfterAttack, PublicRecord,
    },
};

pub type PuctBot<'a, R> = StatefulStrategyBot<State<'a, puct::AttackPUCT<R>>>;

pub struct Params<S: StatefulStrategy> {
    pub first_friendly_troop_reduction: f64,
    pub first_enemy_troop_reduction: f64,
    pub strategy_params: S::Params,
}

pub struct StatefulStrategyBot<S: StatefulStrategy> {
    mcts: S,
    needs_search_reset: bool,
    repeat_move: Option<PlayerMove>,
    first_friendly_troop_reduction: f64,
    first_enemy_troop_reduction: f64,
}

impl<S: StatefulStrategy> StatefulStrategyBot<S> {
    pub fn new(rng: S::Rng) -> Self {
        Self {
            mcts: S::from_rng(rng),
            needs_search_reset: true,
            repeat_move: None,
            first_enemy_troop_reduction: 0.47578774202200713,
            first_friendly_troop_reduction: 0.9711985622851357,
        }
    }

    pub fn with_params(params: Params<S>, rng: S::Rng) -> Self {
        Self {
            mcts: S::with_params(params.strategy_params, rng),
            needs_search_reset: true,
            repeat_move: None,
            first_friendly_troop_reduction: params.first_friendly_troop_reduction,
            first_enemy_troop_reduction: params.first_enemy_troop_reduction,
        }
    }
}

impl<S: StatefulStrategy> ManagedPlayer for StatefulStrategyBot<S> {
    fn reset(&mut self) {
        self.mcts.reset_new();
        self.needs_search_reset = false;
        self.repeat_move = None;
    }

    fn pre_query(&mut self, state: &ClientState, query: &risk_shared::query::Query) {
        if !matches!(
            query.details,
            QueryDetails::Attack | QueryDetails::TroopsAfterAttack(_)
        ) {
            self.needs_search_reset = true;
        }

        if !self.needs_search_reset {
            for (_, record) in query.update.enumerate_items() {
                if let PublicRecord::Attack(r) = record {
                    let PublicRecord::Move(player, Move::Attack(attack)) =
                        state.recording()[r.move_attack_id]
                    else {
                        unreachable!();
                    };

                    if player == state.me().id {
                        let player = PlayerMove {
                            origin: attack.attacking_territory,
                            dest: attack.defending_territory,
                        };

                        self.mcts
                            .make_moves(player, (r.attacking_lost as u8, r.defending_lost as u8));
                    }
                }
            }
        }
    }

    fn query_attack(&mut self, state: &ClientState) -> Option<MoveAttack> {
        if self.needs_search_reset {
            self.needs_search_reset = false;
            let troop_counts = EnumMap::from_fn(|t| state.territories()[t].troops);
            let occupiers = encode_occupiers(state);
            self.mcts
                .reset(troop_counts, occupiers, state.card_sets_redeemed());

            self.repeat_move = None;
        }

        if let Some(repeat_move) = self.repeat_move {
            let PlayerMove { origin, dest } = repeat_move;
            if state.territories()[origin].troops > 1
                && state.territories()[dest].occupier != Some(state.me().id)
            {
                return Some(MoveAttack {
                    attacking_territory: origin,
                    defending_territory: dest,
                    attacking_troops: std::cmp::min(3, state.territories()[origin].troops - 1),
                });
            } else {
                self.repeat_move = None;
            }
        }

        let (mov, repeat) = self.mcts.get_move()?;
        if repeat {
            self.repeat_move = Some(mov);
        }

        Some(MoveAttack {
            attacking_territory: mov.origin,
            defending_territory: mov.dest,
            attacking_troops: std::cmp::min(3, state.territories()[mov.origin].troops - 1),
        })
    }

    fn query_claim_territory(&mut self, state: &ClientState) -> TerritoryId {
        let mut unclaimed_territories = state.territories_owned_by(None);
        let my_territories = state.territories_owned_by(Some(state.me().id));

        let adjacent_territories = util::adjacent_territories(&my_territories);

        // Claim the one with the most of our territories adjacent
        if let Some(selected) = unclaimed_territories
            .iter()
            .filter(|x| adjacent_territories.contains(x))
            .copied()
            .max_by(|&x, &y| {
                let x_adj = EDGES[x]
                    .iter()
                    .filter(|x| my_territories.contains(x))
                    .count();

                let y_adj = EDGES[y]
                    .iter()
                    .filter(|x| my_territories.contains(x))
                    .count();

                // Prefer the one with the highest degree
                x_adj.cmp(&y_adj).then(EDGES[x].len().cmp(&EDGES[y].len()))
            })
        {
            return selected;
        }

        // The default order is very bad, so we reverse first
        unclaimed_territories.reverse();
        let distances = if my_territories.is_empty() {
            EnumMap::from_fn(|_| u32::MAX)
        } else {
            get_territory_distances(&my_territories)
        };

        // Attempt to claim the territories that are closest to our territories
        unclaimed_territories
            .into_iter()
            .min_by(|&x, &y| {
                distances[x]
                    .cmp(&distances[y])
                    .then_with(|| {
                        // Prefer those with the most adjacent unclaimed territories
                        let x_count = EDGES[x]
                            .iter()
                            .filter(|&&x| state.territories()[x].occupier.is_none())
                            .count();

                        let y_count = EDGES[y]
                            .iter()
                            .filter(|&&x| state.territories()[x].occupier.is_none())
                            .count();

                        x_count.cmp(&y_count).reverse()
                    })
                    .then_with(|| {
                        // Prefer those with the fewest adjacent enemy territories
                        let x_count = EDGES[x]
                            .iter()
                            .filter(|&&x| {
                                state.territories()[x]
                                    .occupier
                                    .map_or(false, |x| x != state.me().id)
                            })
                            .count();

                        let y_count = EDGES[y]
                            .iter()
                            .filter(|&&x| {
                                state.territories()[x]
                                    .occupier
                                    .map_or(false, |x| x != state.me().id)
                            })
                            .count();

                        x_count.cmp(&y_count)
                    })
            })
            .unwrap()
    }

    fn query_defend(&mut self, state: &ClientState, move_attack_id: usize) -> MoveDefend {
        let PublicRecord::Move(_, Move::Attack(move_attack)) = state.recording()[move_attack_id]
        else {
            unreachable!();
        };

        let defending_territory = move_attack.defending_territory;
        let defending_troops = std::cmp::min(2, state.territories()[defending_territory].troops);
        MoveDefend {
            move_attack_id,
            defending_troops,
        }
    }

    fn query_distribute_troops(
        &mut self,
        state: &ClientState,
        cause: Cause,
    ) -> MoveDistributeTroops {
        let mut total_troops = state.me().troops_remaining;
        let mut distributions = EnumMap::from_fn(|_| 0);

        if let Some(&t) = state.me().must_place_territory_bonus.first() {
            assert!(total_troops >= 2);
            distributions[t] += 2;
            total_troops -= 2;
        }

        if total_troops > 0 {
            let troop_counts =
                EnumMap::from_fn(|t| state.territories()[t].troops + distributions[t]);
            let occupiers = encode_occupiers(state);
            self.mcts.place_troops(
                total_troops,
                troop_counts,
                occupiers,
                &mut distributions,
                state.card_sets_redeemed(),
            );

            self.needs_search_reset = false;
        }

        MoveDistributeTroops {
            distributions: Box::new(distributions),
            cause,
        }
    }

    fn query_fortify(&mut self, state: &ClientState) -> Option<MoveFortify> {
        // Fortify the non-border territory with most troops to the closest border
        let my_territories = state.territories_owned_by(Some(state.me().id));
        let border_territories = util::border_territories(&my_territories);
        let non_border_territories = util::nonborder_territories(&my_territories);

        let most_troops_territory = non_border_territories
            .iter()
            .copied()
            .max_by_key(|&t| state.territories()[t].troops)?;

        if state.territories()[most_troops_territory].troops <= 1 {
            return None;
        }

        let mut forbidden_set = EnumMap::from_fn(|_| false);
        for t in TerritoryId::ALL {
            if state.territories()[t].occupier != Some(state.me().id) {
                forbidden_set[t] = true;
            }
        }

        let mut target_set = EnumMap::from_fn(|_| false);
        for territory in border_territories {
            target_set[territory] = true;
        }

        find_next_step_to_set(most_troops_territory, forbidden_set, target_set)
            .filter(|&dest| dest != most_troops_territory)
            .map(|dest| MoveFortify {
                source_territory: most_troops_territory,
                target_territory: dest,
                troop_count: state.territories()[most_troops_territory].troops - 1,
            })
    }

    fn query_place_initial_troop(&mut self, state: &ClientState) -> TerritoryId {
        let my_territories = state.territories_owned_by(Some(state.me().id));
        let border_territories = util::border_territories(&my_territories);

        border_territories
            .iter()
            .copied()
            .min_by(|&x, &y| {
                self.territory_strength(state, x)
                    .total_cmp(&self.territory_strength(state, y))
            })
            .unwrap()
    }

    fn query_redeem_cards(&mut self, state: &ClientState, cause: Cause) -> MoveRedeemCards {
        let mut card_sets = Vec::new();
        let mut cards_remaining = state.me().cards.clone();

        while cards_remaining.len() >= 5 {
            let card_set = util::get_card_set(&cards_remaining).unwrap();
            card_sets.push(card_set);
            cards_remaining.retain(|x| !card_set.contains(x));
        }

        if cause.is_turn_started() {
            let territories_held = TerritoryId::ALL
                .into_iter()
                .filter(|&x| state.territories()[x].occupier == Some(state.me().id))
                .count();

            if state.card_sets_redeemed() > 6 || territories_held <= 5 {
                while let Some(card_set) = util::get_card_set(&cards_remaining) {
                    card_sets.push(card_set);
                    cards_remaining.retain(|x| !card_set.contains(x));
                }
            }
        }

        MoveRedeemCards {
            sets: card_sets,
            cause,
        }
    }

    fn query_troops_after_attack(
        &mut self,
        state: &ClientState,
        record_attack_id: usize,
    ) -> MoveTroopsAfterAttack {
        let PublicRecord::Attack(record_attack) = state.recording()[record_attack_id] else {
            unreachable!();
        };

        let PublicRecord::Move(_, Move::Attack(move_attack)) =
            state.recording()[record_attack.move_attack_id]
        else {
            unreachable!();
        };

        MoveTroopsAfterAttack {
            record_attack_id,
            troop_count: state.territories()[move_attack.attacking_territory].troops - 1,
        }
    }
}

impl<S: StatefulStrategy> StatefulStrategyBot<S> {
    fn territory_strength(&self, state: &ClientState, territory: TerritoryId) -> f64 {
        let adjacent_territories = util::adjacent_territories(&[territory]);
        let enemy_strength = adjacent_territories
            .iter()
            .filter(|&&x| state.territories()[x].occupier != Some(state.me().id))
            .map(|&x| state.territories()[x].troops as f64 - self.first_enemy_troop_reduction)
            .sum::<f64>();

        let our_strength =
            state.territories()[territory].troops as f64 - self.first_friendly_troop_reduction;
        let strength = our_strength / enemy_strength;

        if adjacent_territories
            .iter()
            .all(|&x| state.territories()[x].occupier != Some(state.me().id))
        {
            strength + 1000.0
        } else {
            strength
        }
    }
}

fn get_territory_distances(owned: &[TerritoryId]) -> EnumMap<TerritoryId, u32> {
    assert_ne!(owned.len(), 0);
    let mut distances = EnumMap::from_fn(|_| u32::MAX);
    let mut queue = VecDeque::new();

    for &territory in owned {
        distances[territory] = 0;
        queue.push_front(territory);
    }

    while let Some(territory) = queue.pop_back() {
        for &adjacent in EDGES[territory] {
            if distances[adjacent] == u32::MAX {
                distances[adjacent] = distances[territory] + 1;
                queue.push_front(adjacent);
            }
        }
    }

    assert!(distances.values().all(|x| *x != u32::MAX));
    distances
}

fn find_next_step_to_set(
    source: TerritoryId,
    forbidden_set: EnumMap<TerritoryId, bool>,
    target_set: EnumMap<TerritoryId, bool>,
) -> Option<TerritoryId> {
    let mut parent = EnumMap::from_fn(|_| None);
    let mut seen = forbidden_set;
    seen[source] = true;

    let mut queue = VecDeque::new();
    queue.push_back(source);

    let mut found = None;
    while let Some(current) = queue.pop_front() {
        if target_set[current] {
            found = Some(current);
            break;
        }

        for &neighbour in EDGES[current] {
            if !seen[neighbour] {
                seen[neighbour] = true;
                parent[neighbour] = Some(current);
                queue.push_back(neighbour);
            }
        }
    }

    if let Some(mut found) = found {
        let mut last = None;
        while let Some(next) = parent[found] {
            last = Some(found);
            found = next;
        }

        last
    } else {
        None
    }
}

fn encode_occupiers(state: &ClientState) -> EnumMap<TerritoryId, PlayerId> {
    let my_turn_index = state
        .turn_order()
        .iter()
        .position(|&x| x == state.me().id)
        .unwrap();

    let mut player_id_map = EnumMap::from_array([PlayerId::P0; 5]);
    for (i, player) in (0..5)
        .map(|i| state.turn_order()[(my_turn_index + i) % 5])
        .filter(|&player| state.players()[player].alive)
        .enumerate()
    {
        player_id_map[player] = PlayerId::n(i as u8).unwrap();
    }

    EnumMap::from_fn(|t| player_id_map[state.territories()[t].occupier.unwrap()])
}
