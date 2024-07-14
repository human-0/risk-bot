use risk_shared::{
    map::TerritoryId,
    player::{PlayerBot, PlayerId},
    query::{QueryDetails, RecordUpdate},
    record::{
        Cause, Move, MoveAttack, MoveDefend, MoveDistributeTroops, MoveFortify, MoveRedeemCards,
        MoveTroopsAfterAttack,
    },
};

use crate::{censor, state::EngineState};

pub struct PlayerConnection<P: PlayerBot> {
    player: P,
    player_id: PlayerId,
    record_update_watermark: usize,
}

impl<P: PlayerBot> PlayerConnection<P> {
    pub fn new(player: P, player_id: PlayerId) -> Self {
        Self {
            player,
            player_id,
            record_update_watermark: 0,
        }
    }

    pub fn reset(&mut self) {
        self.player.reset();
    }

    pub fn query(&mut self, state: &EngineState, query: QueryDetails) -> Move {
        match query {
            QueryDetails::Attack => self
                .query_attack(state)
                .map_or(Move::AttackPass, Move::Attack),
            QueryDetails::ClaimTerritory => Move::ClaimTerritory(self.query_claim_territory(state)),
            QueryDetails::Defend(move_attack_id) => {
                Move::Defend(self.query_defend(state, move_attack_id))
            }
            QueryDetails::DistributeTroops(cause) => {
                Move::DistributeTroops(self.query_distribute_troops(state, cause))
            }
            QueryDetails::Fortify => self
                .query_fortify(state)
                .map_or(Move::FortifyPass, Move::Fortify),
            QueryDetails::PlaceInitialTroop => {
                Move::PlaceInitialTroop(self.query_place_initial_troop(state))
            }
            QueryDetails::RedeemCards(cause) => {
                Move::RedeemCards(self.query_redeem_cards(state, cause))
            }
            QueryDetails::TroopsAfterAttack(record_attack_id) => {
                Move::MoveTroopsAfterAttack(self.query_troops_after_attack(state, record_attack_id))
            }
        }
    }

    pub fn query_attack(&mut self, state: &EngineState) -> Option<MoveAttack> {
        let update = self.get_record_update(state);
        self.player.query_attack(update)
    }

    pub fn query_claim_territory(&mut self, state: &EngineState) -> TerritoryId {
        let update = self.get_record_update(state);
        self.player.query_claim_territory(update)
    }

    pub fn query_defend(&mut self, state: &EngineState, move_attack_id: usize) -> MoveDefend {
        let update = self.get_record_update(state);
        self.player.query_defend(update, move_attack_id)
    }

    pub fn query_distribute_troops(
        &mut self,
        state: &EngineState,
        cause: Cause,
    ) -> MoveDistributeTroops {
        let update = self.get_record_update(state);
        self.player.query_distribute_troops(update, cause)
    }

    pub fn query_fortify(&mut self, state: &EngineState) -> Option<MoveFortify> {
        let update = self.get_record_update(state);
        self.player.query_fortify(update)
    }

    pub fn query_place_initial_troop(&mut self, state: &EngineState) -> TerritoryId {
        let update = self.get_record_update(state);
        self.player.query_place_initial_troop(update)
    }

    pub fn query_redeem_cards(&mut self, state: &EngineState, cause: Cause) -> MoveRedeemCards {
        let update = self.get_record_update(state);
        self.player.query_redeem_cards(update, cause)
    }

    pub fn query_troops_after_attack(
        &mut self,
        state: &EngineState,
        record_attack_id: usize,
    ) -> MoveTroopsAfterAttack {
        let update = self.get_record_update(state);
        self.player
            .query_troops_after_attack(update, record_attack_id)
    }

    fn get_record_update(&mut self, state: &EngineState) -> RecordUpdate {
        if self.record_update_watermark >= state.recording().len() {
            panic!("Record update watermark out of sync with state");
        }

        let new_records = state
            .recording()
            .iter()
            .skip(self.record_update_watermark)
            .cloned()
            .map(|x| censor::censor(state, x, self.player_id))
            .collect();

        let result = RecordUpdate::new(new_records, self.record_update_watermark);
        self.record_update_watermark = state.recording().len();
        result
    }
}
