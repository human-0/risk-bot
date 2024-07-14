#![cfg_attr(feature = "serde", allow(future_incompatible))]
pub mod map;
pub mod player;
pub mod query;
pub mod record;

#[cfg(feature = "serde")]
pub mod serde;

pub use player::NUM_PLAYERS;

use map::TerritoryId;
use player::PlayerId;

pub const CARD_COUNT: usize = 44;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Card(u8);

impl Card {
    pub fn new(v: u8) -> Option<Self> {
        (v < CARD_COUNT as u8).then_some(Self(v))
    }

    pub fn id(self) -> u8 {
        self.0
    }

    pub fn all() -> [Self; CARD_COUNT] {
        std::array::from_fn(|x| Self::new(x as u8).unwrap())
    }

    pub fn territory(self) -> Option<TerritoryId> {
        (self.0 < 42).then(|| TerritoryId::n(self.0).unwrap())
    }

    pub fn symbol(self) -> CardSymbol {
        CARD_SYMBOLS[self.0 as usize]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, enum_map::Enum)]
pub enum CardSymbol {
    Infantry,
    Cavalry,
    Artillery,
    Wildcard,
}

#[derive(Debug)]
pub struct Territory {
    pub id: TerritoryId,
    pub occupier: Option<PlayerId>,
    pub troops: u32,
}

impl Territory {
    pub fn new(id: TerritoryId) -> Self {
        Self {
            id,
            occupier: None,
            troops: 0,
        }
    }
}

const CARD_SYMBOLS: [CardSymbol; CARD_COUNT] = [
    CardSymbol::Infantry,
    CardSymbol::Cavalry,
    CardSymbol::Artillery,
    CardSymbol::Artillery,
    CardSymbol::Cavalry,
    CardSymbol::Artillery,
    CardSymbol::Cavalry,
    CardSymbol::Cavalry,
    CardSymbol::Artillery,
    CardSymbol::Artillery,
    CardSymbol::Infantry,
    CardSymbol::Artillery,
    CardSymbol::Cavalry,
    CardSymbol::Artillery,
    CardSymbol::Cavalry,
    CardSymbol::Artillery,
    CardSymbol::Cavalry,
    CardSymbol::Infantry,
    CardSymbol::Cavalry,
    CardSymbol::Cavalry,
    CardSymbol::Artillery,
    CardSymbol::Infantry,
    CardSymbol::Infantry,
    CardSymbol::Infantry,
    CardSymbol::Infantry,
    CardSymbol::Cavalry,
    CardSymbol::Cavalry,
    CardSymbol::Cavalry,
    CardSymbol::Infantry,
    CardSymbol::Artillery,
    CardSymbol::Infantry,
    CardSymbol::Infantry,
    CardSymbol::Infantry,
    CardSymbol::Infantry,
    CardSymbol::Infantry,
    CardSymbol::Cavalry,
    CardSymbol::Cavalry,
    CardSymbol::Artillery,
    CardSymbol::Artillery,
    CardSymbol::Infantry,
    CardSymbol::Artillery,
    CardSymbol::Artillery,
    CardSymbol::Wildcard,
    CardSymbol::Wildcard,
];
