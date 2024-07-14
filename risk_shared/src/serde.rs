use std::collections::HashMap;

use crate::{
    map::TerritoryId,
    player::PlayerId,
    query::{Query, QueryDetails, RecordUpdate},
    record::{
        Attack, DrewCard, Move, MoveAttack, MoveDefend, MoveDistributeTroops, MoveFortify,
        MoveRedeemCards, MoveTroopsAfterAttack, PlayerEliminated, PublicPlayerEliminated,
        PublicRecord, PublicStartGame, RedeemedCards, StartTurn, TerritoryConquered,
    },
    Card,
};
use enum_map::EnumMap;
use serde::{
    de::{self, IgnoredAny, Visitor},
    ser::SerializeMap,
    Deserialize, Serialize, Serializer,
};

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "snake_case"))]
enum QueryField {
    QueryType,
    Update,
    Cause,
    MoveAttackId,
    RecordAttackId,
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "snake_case"))]
enum QueryType {
    Attack,
    ClaimTerritory,
    Defend,
    DistributeTroops,
    Fortify,
    PlaceInitialTroop,
    RedeemCards,
    TroopsAfterAttack,
}

impl<'de> Deserialize<'de> for Query {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(QueryVisitor)
    }
}

macro_rules! match_fields {
    ($map:expr, $($type_:pat => $var:ident,)+) => {
        $(
            let mut $var = ::core::option::Option::None;
        )+
        while let ::core::option::Option::Some(key) = ::serde::de::MapAccess::<'_>::next_key(&mut $map)? {
            match key {
                $(
                    $type_ => {
                        if ::core::option::Option::is_some(&$var) {
                            return ::core::result::Result::Err(::serde::de::Error::duplicate_field("query_type"));
                        }

                        $var = ::core::option::Option::Some(::serde::de::MapAccess::<'_>::next_value(&mut $map)?);
                    }
                )+
            }
        }
    };
}

macro_rules! field {
    ($field: expr) => {
        ::core::option::Option::ok_or_else($field, || {
            ::serde::de::Error::missing_field(stringify!($field))
        })?
    };
    ($enum_type:ty, $field:expr) => {{
        let field = field!($field);
        ::core::option::Option::ok_or_else(<$enum_type>::n(field), || {
            ::serde::de::Error::invalid_value(
                ::serde::de::Unexpected::Unsigned(::core::primitive::u64::from(field)),
                &stringify!($enum_type),
            )
        })?
    }};
}

struct QueryVisitor;

impl<'de> Visitor<'de> for QueryVisitor {
    type Value = Query;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("query")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        match_fields! {
            map,
            QueryField::QueryType => query_type,
            QueryField::Update => update,
            QueryField::Cause => cause,
            QueryField::MoveAttackId => move_attack_id,
            QueryField::RecordAttackId => record_attack_id,
        }

        let details = match field!(query_type) {
            QueryType::Attack => QueryDetails::Attack,
            QueryType::ClaimTerritory => QueryDetails::ClaimTerritory,
            QueryType::Defend => QueryDetails::Defend(field!(move_attack_id)),
            QueryType::DistributeTroops => QueryDetails::DistributeTroops(field!(cause)),
            QueryType::Fortify => QueryDetails::Fortify,
            QueryType::PlaceInitialTroop => QueryDetails::PlaceInitialTroop,
            QueryType::RedeemCards => QueryDetails::RedeemCards(field!(cause)),
            QueryType::TroopsAfterAttack => {
                QueryDetails::TroopsAfterAttack(field!(record_attack_id))
            }
        };

        let update = update.ok_or_else(|| de::Error::missing_field("update"))?;

        Ok(Query { details, update })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RecordType {
    RecordAttack,
    RecordDrewCard,
    PublicRecordDrewCard,
    RecordPlayerEliminated,
    // For some reason, this has record_type record_player_eliminated
    // PublicRecordPlayerEliminated,
    RecordRedeemedCards,
    PublicRecordStartGame,
    RecordShuffledCards,
    RecordStartTurn,
    RecordTerritoryConquered,
    MoveAttack,
    MoveAttackPass,
    MoveClaimTerritory,
    MoveDefend,
    MoveDistributeTroops,
    MoveFortify,
    MoveFortifyPass,
    MovePlaceInitialTroop,
    MoveRedeemCards,
    MoveTroopsAfterAttack,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RecordField {
    RecordType,
    MoveAttackId,
    MoveDefendId,
    AttackingTroopsLost,
    DefendingTroopsLost,
    TerritoryConquered,
    DefenderEliminated,
    Player,
    Card,
    RecordAttackId,
    CardsSurrendered,
    CardsSurrenderedCount,
    MoveRedeemCardsId,
    TotalSetBonus,
    MatchingTerritoryBonus,
    TurnOrder,
    Players,
    You,
    ContinentsHeld,
    TerritoriesHeld,
    ContinentBonus,
    TerritoryBonus,

    // Moves
    MoveByPlayer,
    AttackingTerritory,
    DefendingTerritory,
    AttackingTroops,
    Territory,
    DefendingTroops,
    Cause,
    Distributions,
    SourceTerritory,
    TargetTerritory,
    TroopCount,
    Sets,
}

impl<'de> Deserialize<'de> for RecordUpdate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(RecordUpdateVisitor)
    }
}

struct RecordUpdateVisitor;

struct RecordVisitor;

impl<'de> Visitor<'de> for RecordUpdateVisitor {
    type Value = RecordUpdate;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("record update")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let Some(offset) = map.next_key()? else {
            return Ok(RecordUpdate::new(Vec::new(), 0));
        };

        let mut updates = vec![map.next_value()?];
        while let Some((key, value)) = map.next_entry()? {
            let expected_key: usize = updates.len() + offset;
            if key != expected_key {
                let key: usize = key;
                let expected = expected_key.to_string();
                return Err(de::Error::invalid_value(
                    de::Unexpected::Unsigned(key as u64),
                    &expected.as_str(),
                ));
            }

            updates.push(value);
        }

        Ok(RecordUpdate::new(updates, offset))
    }
}

impl<'de> Deserialize<'de> for PublicRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(RecordVisitor)
    }
}

impl<'de> Visitor<'de> for RecordVisitor {
    type Value = PublicRecord;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("record")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        match_fields! {
            map,
            RecordField::RecordType => record_type,
            RecordField::MoveAttackId => move_attack_id,
            RecordField::MoveDefendId => move_defend_id,
            RecordField::AttackingTroopsLost => attacking_troops_lost,
            RecordField::DefendingTroopsLost => defending_troops_lost,
            RecordField::TerritoryConquered => territory_conquered,
            RecordField::DefenderEliminated => defender_eliminated,
            RecordField::Player => player,
            RecordField::Card => card,
            RecordField::RecordAttackId => record_attack_id,
            RecordField::CardsSurrendered => cards_surrendered,
            RecordField::CardsSurrenderedCount => cards_surrendered_count,
            RecordField::MoveRedeemCardsId => move_redeem_cards_id,
            RecordField::TotalSetBonus => total_set_bonus,
            RecordField::MatchingTerritoryBonus => matching_territory_bonus,
            RecordField::TurnOrder => turn_order,
            RecordField::Players => players,
            RecordField::You => you,
            RecordField::ContinentsHeld => continents_held,
            RecordField::TerritoriesHeld => territories_held,
            RecordField::ContinentBonus => continent_bonus,
            RecordField::TerritoryBonus => territory_bonus,
            RecordField::MoveByPlayer => move_by_player,
            RecordField::AttackingTerritory => attacking_territory,
            RecordField::DefendingTerritory => defending_territory,
            RecordField::AttackingTroops => attacking_troops,
            RecordField::Territory => territory,
            RecordField::DefendingTroops => defending_troops,
            RecordField::Cause => cause,
            RecordField::Distributions => distributions,
            RecordField::SourceTerritory => source_territory,
            RecordField::TargetTerritory => target_territory,
            RecordField::TroopCount => troop_count,
            RecordField::Sets => sets,
        }

        let record = match field!(record_type) {
            RecordType::RecordAttack => PublicRecord::Attack(Attack {
                move_attack_id: field!(move_attack_id),
                move_defend_id: field!(move_defend_id),
                attacking_lost: field!(attacking_troops_lost),
                defending_lost: field!(defending_troops_lost),
                territory_conquered: field!(territory_conquered),
                defender_eliminated: field!(defender_eliminated),
            }),
            RecordType::RecordDrewCard => {
                let card = field!(card);
                PublicRecord::DrewCard(DrewCard {
                    player: field!(player),
                    card: field!(card),
                })
            }
            RecordType::PublicRecordDrewCard => PublicRecord::PublicDrewCard(field!(player)),
            RecordType::RecordPlayerEliminated => {
                let player = field!(player);
                let record_attack_id = field!(record_attack_id);
                match (cards_surrendered, cards_surrendered_count) {
                    (Some(cards_surrendered), None) => PublicRecord::PlayerEliminated(PlayerEliminated {
                        player,
                        record_attack_id,
                        cards_surrendered,
                    }),

                    (None, Some(cards_surrendered_count)) => PublicRecord::PublicPlayerEliminated(PublicPlayerEliminated {
                        player,
                        record_attack_id,
                        cards_surrendered_count,
                    }),
                    _ => return Err(de::Error::custom("Exactly one of cards_surrendered and cards_surrendered_count must be present")),
                }
            }
            RecordType::RecordRedeemedCards => PublicRecord::RedeemedCards(RedeemedCards {
                move_id: field!(move_redeem_cards_id),
                total_set_bonus: field!(total_set_bonus),
                matching_territory_bonus: field!(matching_territory_bonus),
            }),
            RecordType::PublicRecordStartGame => {
                PublicRecord::PublicStartGame(Box::new(PublicStartGame {
                    turn_order: field!(turn_order),
                    players: EnumMap::from_array(field!(players)),
                    you: field!(you),
                }))
            }
            RecordType::RecordShuffledCards => PublicRecord::ShuffledCards,
            RecordType::RecordStartTurn => PublicRecord::StartTurn(StartTurn {
                player: field!(player),
                continents_held: field!(continents_held),
                territories_held: field!(territories_held),
                continent_bonus: field!(continent_bonus),
                territory_bonus: field!(territory_bonus),
            }),
            RecordType::RecordTerritoryConquered => {
                PublicRecord::TerritoryConquered(TerritoryConquered {
                    record_attack_id: field!(record_attack_id),
                })
            }
            RecordType::MoveAttack => PublicRecord::Move(
                field!(move_by_player),
                Move::Attack(MoveAttack {
                    attacking_territory: field!(attacking_territory),
                    defending_territory: field!(defending_territory),
                    attacking_troops: field!(attacking_troops),
                }),
            ),
            RecordType::MoveAttackPass => {
                PublicRecord::Move(field!(move_by_player), Move::AttackPass)
            }
            RecordType::MoveClaimTerritory => PublicRecord::Move(
                field!(move_by_player),
                Move::ClaimTerritory(field!(territory)),
            ),
            RecordType::MoveDefend => PublicRecord::Move(
                field!(move_by_player),
                Move::Defend(MoveDefend {
                    move_attack_id: field!(move_attack_id),
                    defending_troops: field!(defending_troops),
                }),
            ),
            RecordType::MoveDistributeTroops => {
                let distributions: HashMap<TerritoryId, u32> = field!(distributions);
                let distributions = EnumMap::from_fn(|t| *distributions.get(&t).unwrap_or(&0));
                PublicRecord::Move(
                    field!(move_by_player),
                    Move::DistributeTroops(MoveDistributeTroops {
                        cause: field!(cause),
                        distributions: Box::new(distributions),
                    }),
                )
            }
            RecordType::MoveFortify => PublicRecord::Move(
                field!(move_by_player),
                Move::Fortify(MoveFortify {
                    source_territory: field!(source_territory),
                    target_territory: field!(target_territory),
                    troop_count: field!(troop_count),
                }),
            ),
            RecordType::MoveFortifyPass => {
                PublicRecord::Move(field!(move_by_player), Move::FortifyPass)
            }
            RecordType::MovePlaceInitialTroop => PublicRecord::Move(
                field!(move_by_player),
                Move::PlaceInitialTroop(field!(territory)),
            ),
            RecordType::MoveRedeemCards => PublicRecord::Move(
                field!(move_by_player),
                Move::RedeemCards(MoveRedeemCards {
                    sets: field!(sets),
                    cause: field!(cause),
                }),
            ),
            RecordType::MoveTroopsAfterAttack => PublicRecord::Move(
                field!(move_by_player),
                Move::MoveTroopsAfterAttack(MoveTroopsAfterAttack {
                    record_attack_id: field!(record_attack_id),
                    troop_count: field!(troop_count),
                }),
            ),
        };

        Ok(record)
    }
}

impl<'de> Deserialize<'de> for Card {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(CardVisitor)
    }
}

struct CardVisitor;

impl<'de> Visitor<'de> for CardVisitor {
    type Value = Card;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("card")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        u8::try_from(v)
            .ok()
            .and_then(Card::new)
            .ok_or_else(|| de::Error::invalid_value(de::Unexpected::Unsigned(v), &"Card"))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<&str>()? {
            if key == "card_id" {
                let value = map.next_value()?;
                let result = Card::new(value).ok_or_else(|| {
                    de::Error::invalid_value(de::Unexpected::Unsigned(u64::from(value)), &"Card")
                })?;

                // Consume remaining keys
                while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                return Ok(result);
            }
        }

        Err(de::Error::missing_field("card_id"))
    }
}

pub struct SerializeMove(pub PlayerId, pub Move);

impl Serialize for SerializeMove {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry(&RecordField::RecordType, &move_record_type(&self.1))?;
        map.serialize_entry(&RecordField::MoveByPlayer, &self.0)?;
        match &self.1 {
            Move::Attack(r) => {
                map.serialize_entry(&RecordField::AttackingTerritory, &r.attacking_territory)?;
                map.serialize_entry(&RecordField::DefendingTerritory, &r.defending_territory)?;
                map.serialize_entry(&RecordField::AttackingTroops, &r.attacking_troops)?;
            }
            Move::AttackPass => (),
            Move::ClaimTerritory(territory) => {
                map.serialize_entry(&RecordField::Territory, &territory)?
            }
            Move::Defend(r) => {
                map.serialize_entry(&RecordField::MoveAttackId, &r.move_attack_id)?;
                map.serialize_entry(&RecordField::DefendingTroops, &r.defending_troops)?;
            }
            Move::DistributeTroops(r) => {
                map.serialize_entry(&RecordField::Cause, &r.cause)?;
                map.serialize_entry(
                    &RecordField::Distributions,
                    &Distributions(*r.distributions),
                )?;
            }
            Move::Fortify(r) => {
                map.serialize_entry(&RecordField::SourceTerritory, &r.source_territory)?;
                map.serialize_entry(&RecordField::TargetTerritory, &r.target_territory)?;
                map.serialize_entry(&RecordField::TroopCount, &r.troop_count)?;
            }
            Move::FortifyPass => (),
            Move::PlaceInitialTroop(territory) => {
                map.serialize_entry(&RecordField::Territory, &territory)?
            }
            Move::RedeemCards(r) => {
                map.serialize_entry(&RecordField::Sets, &Sets(&r.sets))?;
                map.serialize_entry(&RecordField::Cause, &r.cause)?;
            }
            Move::MoveTroopsAfterAttack(r) => {
                map.serialize_entry(&RecordField::RecordAttackId, &r.record_attack_id)?;
                map.serialize_entry(&RecordField::TroopCount, &r.troop_count)?;
            }
        }

        map.end()
    }
}

fn move_record_type(mov: &Move) -> RecordType {
    match mov {
        Move::Attack(_) => RecordType::MoveAttack,
        Move::AttackPass => RecordType::MoveAttackPass,
        Move::ClaimTerritory(_) => RecordType::MoveClaimTerritory,
        Move::Defend(_) => RecordType::MoveDefend,
        Move::DistributeTroops(_) => RecordType::MoveDistributeTroops,
        Move::Fortify(_) => RecordType::MoveFortify,
        Move::FortifyPass => RecordType::MoveFortifyPass,
        Move::PlaceInitialTroop(_) => RecordType::MovePlaceInitialTroop,
        Move::RedeemCards(_) => RecordType::MoveRedeemCards,
        Move::MoveTroopsAfterAttack(_) => RecordType::MoveTroopsAfterAttack,
    }
}

struct Distributions(EnumMap<TerritoryId, u32>);

impl Serialize for Distributions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        for (territory, count) in self.0.iter().filter(|(_, x)| **x != 0) {
            map.serialize_entry(&territory, count)?;
        }

        map.end()
    }
}

struct Sets<'a>(&'a [[Card; 3]]);

impl Serialize for Sets<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.0.iter().map(|x| Set(*x)))
    }
}

struct Set([Card; 3]);

impl Serialize for Set {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.0.iter().map(|x| x.id()))
    }
}
