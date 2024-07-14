use std::ops::DerefMut;

use crate::{
    map::TerritoryId,
    query::{Query, QueryDetails, RecordUpdate},
    record::{
        Cause, Move, MoveAttack, MoveDefend, MoveDistributeTroops, MoveFortify, MoveRedeemCards,
        MoveTroopsAfterAttack,
    },
    Card,
};

pub const NUM_PLAYERS: usize = 5;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, enumn::N, enum_map::Enum)]
#[cfg_attr(
    feature = "serde",
    derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)
)]
pub enum PlayerId {
    P0,
    P1,
    P2,
    P3,
    P4,
}

impl PlayerId {
    pub const ALL: [PlayerId; NUM_PLAYERS] = [
        PlayerId::P0,
        PlayerId::P1,
        PlayerId::P2,
        PlayerId::P3,
        PlayerId::P4,
    ];

    const unsafe fn _count_check() {
        #[allow(clippy::useless_transmute)]
        unsafe {
            std::mem::transmute::<[u8; NUM_PLAYERS], [u8; <Self as enum_map::Enum>::LENGTH]>(
                [0; NUM_PLAYERS],
            );
        }
    }

    /// Returns `true` if the player id is [`P0`].
    ///
    /// [`P0`]: PlayerId::P0
    #[must_use]
    pub fn is_p0(&self) -> bool {
        matches!(self, Self::P0)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct Player {
    #[cfg_attr(feature = "serde", serde(rename = "player_id"))]
    pub id: PlayerId,
    pub troops_remaining: u32,
    pub alive: bool,
    pub cards: Vec<Card>,
    pub must_place_territory_bonus: Vec<TerritoryId>,
}

impl Player {
    pub fn new(id: PlayerId, troops: u32) -> Self {
        Self {
            id,
            troops_remaining: troops,
            alive: true,
            cards: Vec::new(),
            must_place_territory_bonus: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct PublicPlayer {
    #[cfg_attr(feature = "serde", serde(rename = "player_id"))]
    pub id: PlayerId,
    pub troops_remaining: u32,
    pub alive: bool,
    pub card_count: usize,
    pub must_place_territory_bonus: Vec<TerritoryId>,
}

impl PublicPlayer {
    pub fn new(id: PlayerId, troops: u32) -> Self {
        Self {
            id,
            troops_remaining: troops,
            alive: true,
            card_count: 0,
            must_place_territory_bonus: Vec::new(),
        }
    }
}

pub trait PlayerBot {
    fn reset(&mut self);
    fn query(&mut self, query: Query) -> Move;

    fn query_attack(&mut self, update: RecordUpdate) -> Option<MoveAttack> {
        let query = Query {
            details: QueryDetails::Attack,
            update,
        };

        match self.query(query) {
            Move::Attack(m) => Some(m),
            Move::AttackPass => None,
            _ => unreachable!(),
        }
    }

    fn query_claim_territory(&mut self, update: RecordUpdate) -> TerritoryId {
        let query = Query {
            details: QueryDetails::ClaimTerritory,
            update,
        };

        let Move::ClaimTerritory(m) = self.query(query) else {
            unreachable!();
        };

        m
    }

    fn query_defend(&mut self, update: RecordUpdate, move_attack_id: usize) -> MoveDefend {
        let query = Query {
            details: QueryDetails::Defend(move_attack_id),
            update,
        };

        let Move::Defend(m) = self.query(query) else {
            unreachable!();
        };

        m
    }

    fn query_distribute_troops(
        &mut self,
        update: RecordUpdate,
        cause: Cause,
    ) -> MoveDistributeTroops {
        let query = Query {
            details: QueryDetails::DistributeTroops(cause),
            update,
        };

        let Move::DistributeTroops(m) = self.query(query) else {
            unreachable!();
        };

        m
    }

    fn query_fortify(&mut self, update: RecordUpdate) -> Option<MoveFortify> {
        let query = Query {
            details: QueryDetails::Fortify,
            update,
        };

        match self.query(query) {
            Move::Fortify(m) => Some(m),
            Move::FortifyPass => None,
            _ => unreachable!(),
        }
    }

    fn query_place_initial_troop(&mut self, update: RecordUpdate) -> TerritoryId {
        let query = Query {
            details: QueryDetails::PlaceInitialTroop,
            update,
        };

        let Move::PlaceInitialTroop(m) = self.query(query) else {
            unreachable!();
        };

        m
    }

    fn query_redeem_cards(&mut self, update: RecordUpdate, cause: Cause) -> MoveRedeemCards {
        let query = Query {
            details: QueryDetails::RedeemCards(cause),
            update,
        };

        let Move::RedeemCards(m) = self.query(query) else {
            unreachable!();
        };

        m
    }

    fn query_troops_after_attack(
        &mut self,
        update: RecordUpdate,
        record_attack_id: usize,
    ) -> MoveTroopsAfterAttack {
        let query = Query {
            details: QueryDetails::TroopsAfterAttack(record_attack_id),
            update,
        };

        let Move::MoveTroopsAfterAttack(m) = self.query(query) else {
            unreachable!();
        };

        m
    }
}

impl PlayerBot for Box<dyn PlayerBot> {
    fn reset(&mut self) {
        self.deref_mut().reset();
    }

    fn query(&mut self, query: Query) -> Move {
        self.deref_mut().query(query)
    }

    fn query_attack(&mut self, update: RecordUpdate) -> Option<MoveAttack> {
        self.deref_mut().query_attack(update)
    }

    fn query_claim_territory(&mut self, update: RecordUpdate) -> TerritoryId {
        self.deref_mut().query_claim_territory(update)
    }

    fn query_defend(&mut self, update: RecordUpdate, move_attack_id: usize) -> MoveDefend {
        self.deref_mut().query_defend(update, move_attack_id)
    }

    fn query_distribute_troops(
        &mut self,
        update: RecordUpdate,
        cause: Cause,
    ) -> MoveDistributeTroops {
        self.deref_mut().query_distribute_troops(update, cause)
    }

    fn query_fortify(&mut self, update: RecordUpdate) -> Option<MoveFortify> {
        self.deref_mut().query_fortify(update)
    }

    fn query_place_initial_troop(&mut self, update: RecordUpdate) -> TerritoryId {
        self.deref_mut().query_place_initial_troop(update)
    }

    fn query_redeem_cards(&mut self, update: RecordUpdate, cause: Cause) -> MoveRedeemCards {
        self.deref_mut().query_redeem_cards(update, cause)
    }

    fn query_troops_after_attack(
        &mut self,
        update: RecordUpdate,
        record_attack_id: usize,
    ) -> MoveTroopsAfterAttack {
        self.deref_mut()
            .query_troops_after_attack(update, record_attack_id)
    }
}
