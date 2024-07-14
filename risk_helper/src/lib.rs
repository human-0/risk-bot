pub mod state;
pub mod util;

use risk_shared::{
    map::TerritoryId,
    player::PlayerBot,
    query::{Query, QueryDetails},
    record::{
        Cause, Move, MoveAttack, MoveDefend, MoveDistributeTroops, MoveFortify, MoveRedeemCards,
        MoveTroopsAfterAttack,
    },
};
use state::ClientState;

pub trait ManagedPlayer {
    fn reset(&mut self);

    fn pre_query(&mut self, state: &ClientState, query: &Query);

    fn query_attack(&mut self, state: &ClientState) -> Option<MoveAttack>;

    fn query_claim_territory(&mut self, state: &ClientState) -> TerritoryId;

    fn query_defend(&mut self, state: &ClientState, move_attack_id: usize) -> MoveDefend;

    fn query_distribute_troops(
        &mut self,
        state: &ClientState,
        cause: Cause,
    ) -> MoveDistributeTroops;

    fn query_fortify(&mut self, state: &ClientState) -> Option<MoveFortify>;

    fn query_place_initial_troop(&mut self, state: &ClientState) -> TerritoryId;

    fn query_redeem_cards(&mut self, state: &ClientState, cause: Cause) -> MoveRedeemCards;

    fn query_troops_after_attack(
        &mut self,
        state: &ClientState,
        record_attack_id: usize,
    ) -> MoveTroopsAfterAttack;
}

pub struct ManagedPlayerBot<P>
where
    P: ManagedPlayer,
{
    state: ClientState,
    player: P,
}

impl<P> ManagedPlayerBot<P>
where
    P: ManagedPlayer,
{
    pub fn new(player: P) -> Self {
        Self {
            state: ClientState::new(),
            player,
        }
    }
}

impl<P> PlayerBot for ManagedPlayerBot<P>
where
    P: ManagedPlayer,
{
    fn reset(&mut self) {
        self.state = ClientState::new();
        self.player.reset();
    }

    fn query(&mut self, query: Query) -> Move {
        let new_records_mark = self.state.recording().len();
        for (i, record) in query.update.enumerate_items() {
            self.state.commit(i, record.clone());
        }

        self.state.set_new_records(new_records_mark);

        self.player.pre_query(&self.state, &query);
        match query.details {
            QueryDetails::Attack => self
                .player
                .query_attack(&self.state)
                .map_or(Move::AttackPass, Move::Attack),
            QueryDetails::ClaimTerritory => {
                Move::ClaimTerritory(self.player.query_claim_territory(&self.state))
            }
            QueryDetails::Defend(q) => Move::Defend(self.player.query_defend(&self.state, q)),
            QueryDetails::DistributeTroops(q) => {
                Move::DistributeTroops(self.player.query_distribute_troops(&self.state, q))
            }
            QueryDetails::Fortify => self
                .player
                .query_fortify(&self.state)
                .map_or(Move::FortifyPass, Move::Fortify),
            QueryDetails::PlaceInitialTroop => {
                Move::PlaceInitialTroop(self.player.query_place_initial_troop(&self.state))
            }
            QueryDetails::RedeemCards(q) => {
                Move::RedeemCards(self.player.query_redeem_cards(&self.state, q))
            }
            QueryDetails::TroopsAfterAttack(q) => {
                Move::MoveTroopsAfterAttack(self.player.query_troops_after_attack(&self.state, q))
            }
        }
    }
}
