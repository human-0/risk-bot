use enum_map::EnumMap;
use rand::prelude::SliceRandom;

use risk_helper::{
    state::ClientState,
    util::{adjacent_territories, get_card_set},
    ManagedPlayer,
};
use risk_shared::{
    map::{TerritoryId, EDGES},
    query::Query,
    record::{
        Cause, Move, MoveAttack, MoveDefend, MoveDistributeTroops, MoveFortify, MoveRedeemCards,
        MoveTroopsAfterAttack, PublicRecord,
    },
};

pub struct SimpleExample<R: rand::Rng> {
    rng: R,
}

impl<R: rand::Rng> SimpleExample<R> {
    pub fn new(rng: R) -> Self {
        Self { rng }
    }
}

impl<R: rand::Rng> ManagedPlayer for SimpleExample<R> {
    fn reset(&mut self) {}

    fn pre_query(&mut self, _: &ClientState, _: &Query) {}

    fn query_attack(&mut self, state: &ClientState) -> Option<MoveAttack> {
        let my_territories = state.territories_owned_by(Some(state.me().id));
        let bordering_territories = adjacent_territories(&my_territories);

        for target in bordering_territories {
            for &candidate_attacker in EDGES[target].iter().filter(|x| my_territories.contains(x)) {
                if state.territories()[candidate_attacker].troops > 1 {
                    return Some(MoveAttack {
                        attacking_territory: candidate_attacker,
                        defending_territory: target,
                        attacking_troops: std::cmp::min(
                            3,
                            state.territories()[candidate_attacker].troops - 1,
                        ),
                    });
                }
            }
        }

        None
    }

    fn query_claim_territory(&mut self, state: &ClientState) -> TerritoryId {
        let unclaimed_territories = state.territories_owned_by(None);
        *unclaimed_territories.choose(&mut self.rng).unwrap()
    }

    fn query_defend(&mut self, state: &ClientState, move_attack_id: usize) -> MoveDefend {
        let PublicRecord::Move(_, Move::Attack(move_attack)) = &state.recording()[move_attack_id]
        else {
            unreachable!();
        };

        let defending_territory = move_attack.defending_territory;
        let defending_troops = std::cmp::min(state.territories()[defending_territory].troops, 2);
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
        let mut distributions = EnumMap::from_fn(|_| 0);
        let mut total_troops = state.me().troops_remaining;

        if !state.me().must_place_territory_bonus.is_empty() {
            assert!(total_troops >= 2);
            distributions[state.me().must_place_territory_bonus[0]] += 2;
            total_troops -= 2;
        }

        let my_territories = state.territories_owned_by(Some(state.me().id));
        distributions[*my_territories.choose(&mut self.rng).unwrap()] += total_troops;

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
        *my_territories.choose(&mut self.rng).unwrap()
    }

    fn query_redeem_cards(&mut self, state: &ClientState, cause: Cause) -> MoveRedeemCards {
        let mut cards_remaining = state.me().cards.clone();
        let mut card_sets = Vec::new();
        match cause {
            Cause::TurnStarted => {
                while let Some(card_set) = get_card_set(&cards_remaining) {
                    card_sets.push(card_set);
                    cards_remaining.retain(|x| !card_set.contains(x));
                }
            }
            Cause::PlayerEliminated => {
                while cards_remaining.len() >= 5 {
                    let card_set = get_card_set(&cards_remaining).unwrap();
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
            troop_count: state.territories()[move_attack.attacking_territory].troops - 1,
            record_attack_id,
        }
    }
}
