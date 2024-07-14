use enum_map::EnumMap;
use risk_shared::{
    map::TerritoryId,
    player::{Player, PlayerId, PublicPlayer},
    record::PublicRecord,
    Card, Territory,
};

pub mod mutate;

pub struct ClientState {
    deck_card_count: usize,
    discarded_deck: Vec<Card>,
    players: EnumMap<PlayerId, PublicPlayer>,
    territories: EnumMap<TerritoryId, Territory>,
    card_sets_redeemed: u32,
    turn_order: [PlayerId; 5],
    recording: Vec<PublicRecord>,
    new_records: usize,
    me: Player,
}

impl Default for ClientState {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            deck_card_count: 0,
            discarded_deck: Vec::from(Card::all()),
            players: EnumMap::from_fn(|x| PublicPlayer::new(x, 0)),
            territories: EnumMap::from_fn(Territory::new),
            card_sets_redeemed: 0,
            turn_order: PlayerId::ALL,
            recording: Vec::new(),
            new_records: 0,
            me: Player::new(PlayerId::P0, 0),
        }
    }

    pub fn territories_owned_by(&self, player: Option<PlayerId>) -> Vec<TerritoryId> {
        self.territories
            .iter()
            .filter(|(_, x)| x.occupier == player)
            .map(|(x, _)| x)
            .collect()
    }

    pub fn recording(&self) -> &[PublicRecord] {
        &self.recording
    }

    pub fn new_records(&self) -> &[PublicRecord] {
        &self.recording[self.new_records..]
    }

    pub(crate) fn set_new_records(&mut self, new_records: usize) {
        self.new_records = new_records;
    }

    pub fn territories(&self) -> &EnumMap<TerritoryId, Territory> {
        &self.territories
    }

    pub fn me(&self) -> &Player {
        &self.me
    }

    pub fn card_sets_redeemed(&self) -> u32 {
        self.card_sets_redeemed
    }

    pub fn turn_order(&self) -> [PlayerId; 5] {
        self.turn_order
    }

    pub fn players(&self) -> &EnumMap<PlayerId, PublicPlayer> {
        &self.players
    }
}
