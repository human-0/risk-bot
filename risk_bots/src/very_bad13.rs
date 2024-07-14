use enum_map::EnumMap;
use rand::prelude::SliceRandom;
use risk_helper::{state::ClientState, util, ManagedPlayer};
use risk_shared::{
    map::{TerritoryId, EDGES},
    query::Query,
    record::{
        Cause, Move, MoveAttack, MoveDefend, MoveDistributeTroops, MoveFortify, MoveRedeemCards,
        MoveTroopsAfterAttack, PublicRecord,
    },
};

pub struct VeryBad13 {}

impl Default for VeryBad13 {
    fn default() -> Self {
        Self::new()
    }
}

impl VeryBad13 {
    pub fn new() -> Self {
        Self {}
    }
}

impl ManagedPlayer for VeryBad13 {
    fn reset(&mut self) {}

    fn pre_query(&mut self, _: &ClientState, _: &Query) {}

    fn query_attack(&mut self, state: &ClientState) -> Option<MoveAttack> {
        let attack_weakest = |territories: &[TerritoryId]| {
            for &candidate_target in territories {
                let mut candidate_attackers = EDGES[candidate_target]
                    .iter()
                    .copied()
                    .filter(|&x| state.territories()[x].occupier == Some(state.me().id))
                    .collect::<Vec<_>>();

                candidate_attackers
                    .sort_by_key(|&x| std::cmp::Reverse(state.territories()[x].troops));
                if let Some(attacker) = candidate_attackers
                    .into_iter()
                    .find(|&x| state.territories()[x].troops > 1)
                {
                    return Some(MoveAttack {
                        attacking_territory: attacker,
                        defending_territory: candidate_target,
                        attacking_troops: std::cmp::min(
                            3,
                            state.territories()[attacker].troops - 1,
                        ),
                    });
                }
            }

            None
        };

        let mut enemy_territories = state
            .territories()
            .iter()
            .filter(|(_, t)| t.occupier != Some(state.me().id))
            .map(|(id, _)| id)
            .collect::<Vec<_>>();

        enemy_territories.sort_by(|&t1, &t2| {
            enemy_territory_strength(state, t1).total_cmp(&enemy_territory_strength(state, t2))
        });

        attack_weakest(&enemy_territories)
    }

    fn query_claim_territory(&mut self, state: &ClientState) -> TerritoryId {
        let mut unclaimed_territories = state.territories_owned_by(None);

        // Shuffle here to avoid biasing territory selections when testing against this bot
        unclaimed_territories.shuffle(&mut rand::thread_rng());

        let my_territories = state.territories_owned_by(Some(state.me().id));

        let adjacent_territories = util::adjacent_territories(&my_territories);

        // Claim the one with the most of our territories adjacent
        if let Some(selected) = unclaimed_territories
            .iter()
            .filter(|x| adjacent_territories.contains(x))
            .copied()
            .max_by_key(|&x| {
                util::adjacent_territories(&[x])
                    .into_iter()
                    .filter(|x| my_territories.contains(x))
                    .count()
            })
        {
            selected
        } else {
            // Return territory of highest degree
            unclaimed_territories
                .iter()
                .copied()
                .max_by_key(|&x| EDGES[x].len())
                .unwrap()
        }
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
        let my_territories = state.territories_owned_by(Some(state.me().id));
        let border_territories = util::border_territories(&my_territories);

        if let Some(&t) = state.me().must_place_territory_bonus.first() {
            assert!(total_troops >= 2);
            distributions[t] += 2;
            total_troops -= 2;
        }

        let weakest_border_territory = border_territories
            .iter()
            .copied()
            .min_by(|&t1, &t2| {
                my_territory_strength(state, t1).total_cmp(&my_territory_strength(state, t2))
            })
            .unwrap();

        distributions[weakest_border_territory] += total_troops;

        MoveDistributeTroops {
            distributions: Box::new(distributions),
            cause,
        }
    }

    fn query_fortify(&mut self, _: &ClientState) -> Option<MoveFortify> {
        None
    }

    fn query_place_initial_troop(&mut self, state: &ClientState) -> TerritoryId {
        let my_territories = state.territories_owned_by(Some(state.me().id));
        let border_territories = util::border_territories(&my_territories);

        border_territories
            .iter()
            .copied()
            .min_by(|&t1, &t2| {
                my_territory_strength(state, t1).total_cmp(&my_territory_strength(state, t2))
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

        if state.card_sets_redeemed() > 12 && cause.is_turn_started() {
            while let Some(card_set) = util::get_card_set(&cards_remaining) {
                card_sets.push(card_set);
                cards_remaining.retain(|x| !card_set.contains(x));
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

fn my_territory_strength(state: &ClientState, territory: TerritoryId) -> f64 {
    let ours = EDGES[territory]
        .iter()
        .chain([territory, territory].iter())
        .filter(|&&t| state.territories()[t].occupier == Some(state.me().id))
        .map(|&t| state.territories()[t].troops)
        .sum::<u32>();

    let theirs = EDGES[territory]
        .iter()
        .chain([territory, territory].iter())
        .filter(|&&t| state.territories()[t].occupier != Some(state.me().id))
        .map(|&t| state.territories()[t].troops)
        .sum::<u32>();

    ours as f64 / theirs as f64
}

fn enemy_territory_strength(state: &ClientState, territory: TerritoryId) -> f64 {
    let ours = EDGES[territory]
        .iter()
        .chain([territory, territory].iter())
        .filter(|&&t| state.territories()[t].occupier == Some(state.me().id))
        .map(|&t| state.territories()[t].troops)
        .sum::<u32>();

    let theirs = EDGES[territory]
        .iter()
        .chain([territory, territory].iter())
        .filter(|&&t| state.territories()[t].occupier != Some(state.me().id))
        .map(|&t| state.territories()[t].troops)
        .sum::<u32>();

    theirs as f64 / ours as f64
}
