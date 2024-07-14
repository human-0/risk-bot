use risk_shared::map::TerritoryId;

/// For now, we always move the maximum number of troops possible
/// and assume that the opponent defends the same way
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(C)]
pub struct PlayerMove {
    pub origin: TerritoryId,
    pub dest: TerritoryId,
}

fn _size_check() {
    unsafe {
        std::mem::transmute::<[u8; 2], PlayerMove>([0_u8; 2]);
    }
}

#[repr(align(4))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Move {
    Player(PlayerMove),
    Chance(u8, u8),
    PlaceTroops(TerritoryId),
}

impl Move {
    /// Returns `true` if the move is [`Player`].
    ///
    /// [`Player`]: Move::Player
    #[must_use]
    pub fn is_player(&self) -> bool {
        matches!(self, Self::Player(..))
    }

    /// Returns `true` if the move is [`Chance`].
    ///
    /// [`Chance`]: Move::Chance
    #[must_use]
    pub fn is_chance(&self) -> bool {
        matches!(self, Self::Chance(..))
    }
}
