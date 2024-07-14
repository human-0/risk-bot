pub mod mutate;
pub mod record;

use enum_map::EnumMap;
use risk_shared::record::Record;
use risk_shared::{
    map::TerritoryId,
    player::{Player, PlayerId},
    Card, Territory,
};

use crate::NUM_STARTING_TROOPS;

pub struct EngineState {
    deck: Vec<Card>,
    discarded_deck: Vec<Card>,
    players: EnumMap<PlayerId, Player>,
    territories: EnumMap<TerritoryId, Territory>,
    card_sets_redeemed: u32,
    turn_order: [PlayerId; 5],
    recording: Vec<Record>,
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineState {
    pub fn new() -> Self {
        Self {
            deck: Vec::new(),
            discarded_deck: Vec::from(Card::all()),
            players: EnumMap::from_fn(|id| Player::new(id, NUM_STARTING_TROOPS)),
            territories: EnumMap::from_fn(Territory::new),
            card_sets_redeemed: 0,
            turn_order: [
                PlayerId::P0,
                PlayerId::P1,
                PlayerId::P2,
                PlayerId::P3,
                PlayerId::P4,
            ],
            recording: Vec::new(),
        }
    }

    pub fn recording(&self) -> &[Record] {
        &self.recording
    }

    pub fn territories(&self) -> &EnumMap<TerritoryId, Territory> {
        &self.territories
    }

    pub fn turn_order(&self) -> [PlayerId; 5] {
        self.turn_order
    }

    pub fn players(&self) -> &EnumMap<PlayerId, Player> {
        &self.players
    }

    fn draw_card(&mut self) -> Card {
        self.deck
            .pop()
            .expect("Need to shuffle deck before drawing")
    }

    pub fn deck(&self) -> &[Card] {
        &self.deck
    }
}
