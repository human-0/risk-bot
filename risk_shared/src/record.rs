mod mov;

use crate::{
    map::Continent,
    player::{Player, PlayerId, PublicPlayer},
    Card,
};
use enum_map::EnumMap;
pub use mov::*;

#[derive(Clone, Debug)]
pub enum Record {
    Attack(Attack),
    DrewCard(DrewCard),
    PlayerEliminated(PlayerEliminated),
    RedeemedCards(RedeemedCards),
    ShuffledCards,
    StartGame(Box<StartGame>),
    StartTurn(StartTurn),
    TerritoryConquered(TerritoryConquered),
    Winner(PlayerId),
    Move(PlayerId, Move),
}

#[derive(Clone, Debug)]
pub enum PublicRecord {
    Attack(Attack),
    DrewCard(DrewCard),
    PublicDrewCard(PlayerId),
    PlayerEliminated(PlayerEliminated),
    PublicPlayerEliminated(PublicPlayerEliminated),
    RedeemedCards(RedeemedCards),
    ShuffledCards,
    PublicStartGame(Box<PublicStartGame>),
    StartTurn(StartTurn),
    TerritoryConquered(TerritoryConquered),
    Winner(PlayerId),
    Move(PlayerId, Move),
}

impl Record {
    pub fn censor(self, player: PlayerId) -> PublicRecord {
        match self {
            Record::Attack(v) => PublicRecord::Attack(v),
            Record::DrewCard(v) => PublicRecord::DrewCard(v),
            Record::PlayerEliminated(v) => PublicRecord::PlayerEliminated(v),
            Record::RedeemedCards(v) => PublicRecord::RedeemedCards(v),
            Record::ShuffledCards => PublicRecord::ShuffledCards,
            Record::StartGame(v) => PublicRecord::PublicStartGame(Box::new(v.censor(player))),
            Record::StartTurn(v) => PublicRecord::StartTurn(v),
            Record::TerritoryConquered(v) => PublicRecord::TerritoryConquered(v),
            Record::Winner(v) => PublicRecord::Winner(v),
            Record::Move(p, v) => PublicRecord::Move(p, v),
        }
    }

    pub fn move_attack(player_id: PlayerId, attack: Option<MoveAttack>) -> Self {
        Record::Move(player_id, attack.map_or(Move::AttackPass, Move::Attack))
    }

    pub fn move_fortify(player_id: PlayerId, attack: Option<MoveFortify>) -> Self {
        Record::Move(player_id, attack.map_or(Move::FortifyPass, Move::Fortify))
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Cause {
    TurnStarted,
    PlayerEliminated,
}

impl Cause {
    /// Returns `true` if the cause is [`TurnStarted`].
    ///
    /// [`TurnStarted`]: Cause::TurnStarted
    #[must_use]
    pub fn is_turn_started(&self) -> bool {
        matches!(self, Self::TurnStarted)
    }

    /// Returns `true` if the cause is [`PlayerEliminated`].
    ///
    /// [`PlayerEliminated`]: Cause::PlayerEliminated
    #[must_use]
    pub fn is_player_eliminated(&self) -> bool {
        matches!(self, Self::PlayerEliminated)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Attack {
    pub move_attack_id: usize,
    pub move_defend_id: usize,
    pub attacking_lost: u32,
    pub defending_lost: u32,
    pub territory_conquered: bool,
    pub defender_eliminated: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct DrewCard {
    pub player: PlayerId,
    pub card: Card,
}

#[derive(Clone, Debug)]
pub struct PlayerEliminated {
    pub player: PlayerId,
    pub record_attack_id: usize,
    pub cards_surrendered: Vec<Card>,
}

#[derive(Clone, Copy, Debug)]
pub struct PublicPlayerEliminated {
    pub player: PlayerId,
    pub record_attack_id: usize,
    pub cards_surrendered_count: usize,
}

#[derive(Clone, Debug)]
pub struct RedeemedCards {
    pub move_id: usize,
    pub total_set_bonus: u32,
    pub matching_territory_bonus: u32,
}

#[derive(Clone, Debug)]
pub struct StartGame {
    pub turn_order: [PlayerId; 5],
    pub players: EnumMap<PlayerId, Player>,
}

impl StartGame {
    pub fn censor(&self, player: PlayerId) -> PublicStartGame {
        let you = self.players[player].clone();
        PublicStartGame {
            turn_order: self.turn_order,
            players: EnumMap::from_fn(|id| PublicPlayer {
                id,
                troops_remaining: self.players[id].troops_remaining,
                alive: self.players[id].alive,
                must_place_territory_bonus: self.players[id].must_place_territory_bonus.clone(),
                card_count: self.players[id].cards.len(),
            }),
            you,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PublicStartGame {
    pub turn_order: [PlayerId; 5],
    pub players: EnumMap<PlayerId, PublicPlayer>,
    pub you: Player,
}

#[derive(Clone, Debug)]
pub struct StartTurn {
    pub player: PlayerId,
    pub continents_held: Vec<Continent>,
    pub territories_held: u32,
    pub continent_bonus: u32,
    pub territory_bonus: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct TerritoryConquered {
    pub record_attack_id: usize,
}
