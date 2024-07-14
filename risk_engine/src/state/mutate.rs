use std::collections::HashSet;

use rand::prelude::SliceRandom;

use risk_shared::{
    map::TerritoryId,
    player::PlayerId,
    record::{
        Attack, DrewCard, Move, MoveAttack, MoveDefend, MoveDistributeTroops, MoveFortify,
        MoveRedeemCards, MoveTroopsAfterAttack, PlayerEliminated, Record, RedeemedCards, StartGame,
        StartTurn, TerritoryConquered,
    },
    Card,
};

use super::EngineState;

impl EngineState {
    pub fn commit(&mut self, record: Record) {
        self.recording.push(record.clone());
        match record {
            Record::Attack(r) => self.commit_record_attack(r),
            Record::DrewCard(r) => self.commit_record_drew_card(r),
            Record::PlayerEliminated(r) => self.commit_record_player_eliminated(r),
            Record::RedeemedCards(r) => self.commit_record_redeemed_cards(r),
            Record::ShuffledCards => self.commit_record_shuffled_cards(),
            Record::StartGame(r) => self.commit_record_start_game(*r),
            Record::StartTurn(r) => self.commit_record_start_turn(r),
            Record::TerritoryConquered(r) => self.commit_record_territory_conquered(r),
            Record::Winner(player) => self.commit_record_winner(player),
            Record::Move(player, mov) => match mov {
                Move::Attack(r) => self.commit_move_attack(player, r),
                Move::AttackPass => self.commit_move_attack_pass(player),
                Move::ClaimTerritory(territory) => {
                    self.commit_move_claim_territory(player, territory)
                }
                Move::Defend(r) => self.commit_move_defend(player, r),
                Move::DistributeTroops(r) => self.commit_move_distribute_troops(player, r),
                Move::Fortify(r) => self.commit_move_fortify(player, r),
                Move::FortifyPass => self.commit_move_fortify_pass(player),
                Move::PlaceInitialTroop(territory) => {
                    self.commit_move_place_initial_troop(player, territory)
                }
                Move::RedeemCards(r) => self.commit_move_redeem_cards(player, r),
                Move::MoveTroopsAfterAttack(r) => self.commit_move_troops_after_attack(player, r),
            },
        }
    }

    fn commit_move_attack(&mut self, _: PlayerId, _: MoveAttack) {}

    fn commit_move_attack_pass(&mut self, _: PlayerId) {}

    fn commit_move_claim_territory(&mut self, player: PlayerId, territory: TerritoryId) {
        let claimed_territory = &mut self.territories[territory];
        claimed_territory.occupier = Some(player);
        claimed_territory.troops = 1;

        self.players[player].troops_remaining -= 1;
    }

    fn commit_move_defend(&mut self, _: PlayerId, _: MoveDefend) {}

    fn commit_move_distribute_troops(&mut self, player: PlayerId, r: MoveDistributeTroops) {
        let player = &mut self.players[player];
        player.troops_remaining = 0;

        player.must_place_territory_bonus.clear();

        for (territory, troops) in r.distributions.into_iter() {
            self.territories[territory].troops += troops;
        }
    }

    fn commit_move_fortify(&mut self, _: PlayerId, r: MoveFortify) {
        self.territories[r.source_territory].troops -= r.troop_count;
        self.territories[r.target_territory].troops += r.troop_count;
    }

    fn commit_move_fortify_pass(&mut self, _: PlayerId) {}

    fn commit_move_place_initial_troop(&mut self, player: PlayerId, territory: TerritoryId) {
        self.territories[territory].troops += 1;
        self.players[player].troops_remaining -= 1;
    }

    fn commit_move_redeem_cards(&mut self, player: PlayerId, r: MoveRedeemCards) {
        fn calculate_set_bonus(x: u32) -> u32 {
            const FIXED_VALUES: [u32; 6] = [4, 6, 8, 10, 12, 15];
            FIXED_VALUES
                .get(x as usize)
                .copied()
                .unwrap_or_else(|| 15 + (x - FIXED_VALUES.len() as u32 + 1) * 5)
        }

        let total_set_bonus = {
            let mut set_bonus = 0;
            for _ in 0..r.sets.len() {
                set_bonus += calculate_set_bonus(self.card_sets_redeemed);
                self.card_sets_redeemed += 1;
            }

            set_bonus
        };

        let all_cards = r.sets.iter().copied().flatten().collect::<Vec<_>>();
        let matching_territories = all_cards
            .iter()
            .copied()
            .filter_map(Card::territory)
            .filter(|&x| self.territories[x].occupier == Some(player))
            .collect::<HashSet<_>>();

        let matching_territory_bonus = if matching_territories.is_empty() {
            0
        } else {
            2
        };

        let player = &mut self.players[player];
        player.troops_remaining += total_set_bonus + matching_territory_bonus;
        player.must_place_territory_bonus = matching_territories.into_iter().collect();
        player.cards.retain(|card| !all_cards.contains(card));

        self.discarded_deck.extend(all_cards);

        let record = Record::RedeemedCards(RedeemedCards {
            move_id: self.recording.len() - 1,
            total_set_bonus,
            matching_territory_bonus,
        });

        self.commit(record)
    }

    fn commit_move_troops_after_attack(&mut self, _: PlayerId, r: MoveTroopsAfterAttack) {
        let Record::Attack(attack) = &self.recording[r.record_attack_id] else {
            unreachable!();
        };

        let Record::Move(_, Move::Attack(move_attack)) = &self.recording[attack.move_attack_id]
        else {
            unreachable!();
        };

        self.territories[move_attack.attacking_territory].troops -= r.troop_count;
        self.territories[move_attack.defending_territory].troops += r.troop_count;
    }

    fn commit_record_attack(&mut self, r: Attack) {
        let Record::Move(player, Move::Attack(move_attack)) = &self.recording[r.move_attack_id]
        else {
            unreachable!();
        };

        self.territories[move_attack.attacking_territory].troops -= r.attacking_lost;
        self.territories[move_attack.defending_territory].troops -= r.defending_lost;

        if r.territory_conquered {
            self.territories[move_attack.defending_territory].occupier = Some(*player);
        }
    }

    fn commit_record_drew_card(&mut self, r: DrewCard) {
        self.players[r.player].cards.push(r.card);
    }

    fn commit_record_player_eliminated(&mut self, r: PlayerEliminated) {
        self.players[r.player].alive = false;

        let Record::Attack(attack) = &self.recording[r.record_attack_id] else {
            unreachable!();
        };

        let Record::Move(player, Move::Attack(_)) = &self.recording[attack.move_attack_id] else {
            unreachable!();
        };
        self.players[*player].cards.extend(r.cards_surrendered)
    }

    fn commit_record_redeemed_cards(&mut self, _: RedeemedCards) {}

    fn commit_record_shuffled_cards(&mut self) {
        assert!(
            self.deck.is_empty(),
            "Shuffled cards before deck was empty."
        );

        self.deck = std::mem::take(&mut self.discarded_deck);
        self.deck.shuffle(&mut rand::thread_rng());
    }

    fn commit_record_start_game(&mut self, r: StartGame) {
        self.turn_order = r.turn_order;
    }

    fn commit_record_start_turn(&mut self, r: StartTurn) {
        self.players[r.player].troops_remaining += r.territory_bonus + r.continent_bonus;
    }

    fn commit_record_territory_conquered(&mut self, _: TerritoryConquered) {}

    fn commit_record_winner(&mut self, _: PlayerId) {}
}
