use enum_map::EnumMap;

use crate::{map::TerritoryId, record::Cause, Card};

#[derive(Clone, Debug)]
pub enum Move {
    Attack(MoveAttack),
    AttackPass,
    ClaimTerritory(TerritoryId),
    Defend(MoveDefend),
    DistributeTroops(MoveDistributeTroops),
    Fortify(MoveFortify),
    FortifyPass,
    PlaceInitialTroop(TerritoryId),
    RedeemCards(MoveRedeemCards),
    MoveTroopsAfterAttack(MoveTroopsAfterAttack),
}

#[derive(Clone, Copy, Debug)]
pub struct MoveAttack {
    pub attacking_territory: TerritoryId,
    pub defending_territory: TerritoryId,
    pub attacking_troops: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct MoveDefend {
    pub move_attack_id: usize,
    pub defending_troops: u32,
}

#[derive(Clone, Debug)]
pub struct MoveDistributeTroops {
    pub cause: Cause,
    pub distributions: Box<EnumMap<TerritoryId, u32>>,
}

#[derive(Clone, Copy, Debug)]
pub struct MoveFortify {
    pub source_territory: TerritoryId,
    pub target_territory: TerritoryId,
    pub troop_count: u32,
}

#[derive(Clone, Debug)]
pub struct MoveRedeemCards {
    pub sets: Vec<[Card; 3]>,
    pub cause: Cause,
}

#[derive(Clone, Copy, Debug)]
pub struct MoveTroopsAfterAttack {
    pub record_attack_id: usize,
    pub troop_count: u32,
}
