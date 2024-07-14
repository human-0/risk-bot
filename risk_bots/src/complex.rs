use enum_map::EnumMap;
use rand::prelude::SliceRandom;
use risk_helper::{state::ClientState, util, ManagedPlayer};
use risk_shared::{
    map::{TerritoryId, EDGES},
    player::PlayerId,
    query::Query,
    record::{
        Cause, Move, MoveAttack, MoveDefend, MoveDistributeTroops, MoveFortify, MoveRedeemCards,
        MoveTroopsAfterAttack, PublicRecord,
    },
};

pub struct ComplexExample<R: rand::Rng + Clone> {
    rng: R,
    enemy: Option<PlayerId>,
}

impl<R: rand::Rng + Clone> ComplexExample<R> {
    pub fn new(rng: R) -> Self {
        Self { rng, enemy: None }
    }
}

impl<R: rand::Rng + Clone> ManagedPlayer for ComplexExample<R> {
    fn reset(&mut self) {
        self.enemy = None;
    }

    fn pre_query(&mut self, _: &ClientState, _: &Query) {}

    fn query_attack(&mut self, state: &ClientState) -> Option<MoveAttack> {
        let my_territories = state.territories_owned_by(Some(state.me().id));
        let bordering_territories = util::adjacent_territories(&my_territories);

        let attack_weakest = |territories: &[TerritoryId]| {
            let mut territories = Vec::from(territories);
            territories.sort_by_key(|&x| state.territories()[x].troops);

            for candidate_target in territories {
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

        if state.recording().len() < 4000 {
            let mut enemy = None;
            for record in state.new_records() {
                if let PublicRecord::Move(player, Move::Attack(r)) = record {
                    if state.territories()[r.defending_territory].occupier == Some(state.me().id) {
                        enemy = Some(player)
                    }
                }
            }

            if let Some(enemy) = enemy {
                if self.enemy.is_none() || self.rng.gen_bool(0.05) {
                    self.enemy = Some(*enemy);
                }
            } else {
                let weakest_territory = bordering_territories
                    .iter()
                    .copied()
                    .min_by_key(|&x| state.territories()[x].troops)
                    .unwrap();

                self.enemy = state.territories()[weakest_territory].occupier;
            }

            if let Some(mov) = attack_weakest(&bordering_territories) {
                return Some(mov);
            }

            if self.rng.gen_bool(0.8) {
                if let Some(mov) = attack_weakest(&bordering_territories) {
                    return Some(mov);
                }
            }
        } else {
            let mut strongest_territories = my_territories;
            strongest_territories
                .sort_by_key(|&x| std::cmp::Reverse(state.territories()[x].troops));

            for territory in strongest_territories {
                let adjacent = EDGES[territory]
                    .iter()
                    .copied()
                    .filter(|&x| state.territories()[x].occupier != Some(state.me().id))
                    .collect::<Vec<_>>();

                if let Some(mov) = attack_weakest(&adjacent) {
                    return Some(mov);
                }
            }
        }

        None
    }

    fn query_claim_territory(&mut self, state: &ClientState) -> TerritoryId {
        let mut unclaimed_territories = state.territories_owned_by(None);

        // Shuffle here to avoid biasing territory selections when testing against this bot
        unclaimed_territories.shuffle(&mut self.rng);

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

        if state.recording().len() < 4000 {
            let troops_per_territory = total_troops / border_territories.len() as u32;
            let leftover_troops = total_troops % border_territories.len() as u32;
            for &territory in &border_territories {
                distributions[territory] += troops_per_territory;
            }

            distributions[border_territories[0]] += leftover_troops;
        } else {
            let mut weakest_players = state
                .players()
                .values()
                .filter(|x| x.id != state.me().id)
                .collect::<Vec<_>>();

            weakest_players.sort_by_cached_key(|player| {
                state
                    .territories()
                    .values()
                    .filter(|x| x.occupier == Some(player.id))
                    .map(|x| x.troops)
                    .sum::<u32>()
            });

            for player in weakest_players {
                if let Some(bordering_enemy_territory) = util::adjacent_territories(&my_territories)
                    .into_iter()
                    .find(|&x| state.territories()[x].occupier == Some(player.id))
                {
                    let selected_territory =
                        util::adjacent_territories(&[bordering_enemy_territory])
                            .into_iter()
                            .find(|x| state.territories()[*x].occupier == Some(state.me().id))
                            .unwrap();

                    distributions[selected_territory] += total_troops;
                    break;
                }
            }
        }

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
            .min_by_key(|&x| state.territories()[x].troops)
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
