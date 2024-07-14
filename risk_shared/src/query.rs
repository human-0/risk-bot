use std::ops::Index;

use crate::record::{Cause, PublicRecord};

pub struct Query {
    pub details: QueryDetails,
    pub update: RecordUpdate,
}

pub enum QueryDetails {
    Attack,
    ClaimTerritory,
    Defend(usize),
    DistributeTroops(Cause),
    Fortify,
    PlaceInitialTroop,
    RedeemCards(Cause),
    TroopsAfterAttack(usize),
}

pub struct RecordUpdate {
    updates: Vec<PublicRecord>,
    offset: usize,
}

impl RecordUpdate {
    pub fn new(updates: Vec<PublicRecord>, offset: usize) -> Self {
        Self { updates, offset }
    }

    pub fn enumerate_items(&self) -> impl Iterator<Item = (usize, &PublicRecord)> {
        self.updates
            .iter()
            .enumerate()
            .map(|(i, r)| (i + self.offset, r))
    }
}

impl Index<usize> for RecordUpdate {
    type Output = PublicRecord;

    fn index(&self, index: usize) -> &Self::Output {
        &self.updates[index
            .checked_sub(self.offset)
            .expect("Record update not found")]
    }
}
