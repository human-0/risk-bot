pub mod mov;
use std::cmp;

use enum_map::EnumMap;
pub use mov::*;

use risk_shared::{
    map::{TerritoryId, EDGES},
    player::PlayerId,
};

#[derive(Clone, Debug)]
pub struct AttackGame {
    troops: EnumMap<TerritoryId, u32>,
    occupiers: EnumMap<TerritoryId, PlayerId>, // Player ids based on turn order
    turn: Turn,
    territory_conquered: bool,
    players_eliminated: u8,
    troops_to_place: u32,
    card_sets_redeemed: u32,
}

impl Default for AttackGame {
    fn default() -> Self {
        Self::new()
    }
}

impl AttackGame {
    pub fn new() -> Self {
        Self {
            troops: EnumMap::from_fn(|_| 1),
            occupiers: EnumMap::from_fn(|_| PlayerId::P0),
            turn: Turn::Player,
            territory_conquered: false,
            troops_to_place: 0,
            players_eliminated: 0,
            card_sets_redeemed: 0,
        }
    }

    pub fn set_state(
        &mut self,
        troop_counts: EnumMap<TerritoryId, u32>,
        occupiers: EnumMap<TerritoryId, PlayerId>,
        card_sets_redeemed: u32,
    ) {
        self.troops = troop_counts;
        self.occupiers = occupiers;
        self.card_sets_redeemed = card_sets_redeemed;

        self.territory_conquered = false;
        self.players_eliminated = 0;
    }

    pub fn set_troops_to_place(&mut self, troops: u32) {
        self.turn = Turn::PlaceTroops;
        self.troops_to_place = troops;
    }

    pub fn make_move(&mut self, mov: Move) {
        match mov {
            Move::Player(mov) => self.make_player_move(mov),
            Move::Chance(attackers_lost, defenders_lost) => {
                self.make_chance_move(attackers_lost as u32, defenders_lost as u32)
            }
            Move::PlaceTroops(territory) => {
                assert!(self.occupiers[territory] == PlayerId::P0);
                let troop_count = 20_u32.clamp(self.troops_to_place / 5, self.troops_to_place);
                self.troops[territory] += troop_count;
                self.troops_to_place -= troop_count;

                if self.troops_to_place == 0 {
                    self.turn = Turn::Player;
                }
            }
        }
    }

    fn make_player_move(&mut self, mov: PlayerMove) {
        assert!(self.turn.is_player());
        self.turn = Turn::Chance(mov);
    }

    pub(crate) fn make_chance_move(&mut self, attackers_lost: u32, defenders_lost: u32) {
        let Turn::Chance(mov) = self.turn else {
            unreachable!()
        };

        assert!(self.troops[mov.origin] > attackers_lost);
        assert!(self.troops[mov.dest] >= defenders_lost);
        assert_eq!(self.occupiers[mov.origin], PlayerId::P0);
        assert_ne!(self.occupiers[mov.dest], PlayerId::P0);

        self.troops[mov.origin] -= attackers_lost;
        self.troops[mov.dest] -= defenders_lost;

        // Always move all troops when a territory is conquered
        if self.troops[mov.dest] == 0 {
            self.territory_conquered = true;

            let troops_to_move = self.troops[mov.origin] - 1;
            assert_ne!(troops_to_move, 0);

            self.troops[mov.origin] = 1;
            self.troops[mov.dest] = troops_to_move;

            let last_occupier = self.occupiers[mov.dest];
            self.occupiers[mov.dest] = PlayerId::P0;

            if !self.occupiers.values().any(|&x| x == last_occupier) {
                self.players_eliminated += 1;
            }
        }

        self.turn = Turn::Player;
    }

    pub fn territories_occupied(&self) -> usize {
        self.occupiers.values().filter(|x| x.is_p0()).count()
    }

    pub fn players_remaining(&self) -> usize {
        let mut alive = EnumMap::from_fn(|_| false);
        for &player in self.occupiers.values() {
            alive[player] = true;
        }

        alive.values().filter(|&&x| x).count()
    }

    pub fn initial_players(&self) -> usize {
        self.players_remaining() + self.players_eliminated as usize
    }

    pub fn gen_player_moves_into(&self, move_list: &mut Vec<PlayerMove>) {
        move_list.clear();
        assert_eq!(self.turn, Turn::Player);
        for origin in self
            .troops
            .iter()
            .filter(|(t, x)| self.occupiers[*t].is_p0() && **x > 1)
            .map(|(x, _)| x)
        {
            for &dest in EDGES[origin]
                .iter()
                .filter(|&&t| !self.occupiers[t].is_p0())
            {
                move_list.push(PlayerMove { origin, dest });
            }
        }
    }

    pub fn gen_chance_moves_into(&self, move_list: &mut Vec<(u8, u8)>) {
        move_list.clear();
        let Turn::Chance(mov) = self.turn else {
            unreachable!();
        };

        let origin_troops = self.troops[mov.origin];
        let dest_troops = self.troops[mov.dest];

        assert_eq!(self.occupiers[mov.origin], PlayerId::P0);
        assert_ne!(self.occupiers[mov.dest], PlayerId::P0);

        let num_attackers = cmp::min(origin_troops - 1, 3);
        let num_defenders = cmp::min(dest_troops, 2);
        let battles = cmp::min(num_attackers, num_defenders);

        for count in 0..=battles {
            move_list.push((count as u8, (battles - count) as u8));
        }
    }

    pub fn gen_player_moves_incremental(
        &self,
        last_move: PlayerMove,
        move_list: &mut Vec<PlayerMove>,
    ) {
        assert!(self.turn().is_player());

        let one_troop_remaining = self.troops[last_move.origin] == 1;
        let territory_captured = self.occupiers[last_move.dest].is_p0();
        if territory_captured {
            if one_troop_remaining {
                retain_different_origin_dest(move_list, last_move.origin, last_move.dest);
            } else {
                retain_different_dest(move_list, last_move.dest);
            }

            // Add new moves from conquered territory
            if self.troops[last_move.dest] > 1 {
                for &dest in EDGES[last_move.dest]
                    .iter()
                    .filter(|&&x| !self.occupiers[x].is_p0())
                {
                    move_list.push(PlayerMove {
                        origin: last_move.dest,
                        dest,
                    });
                }
            }
        } else if one_troop_remaining {
            // Can no longer attack from the same territory
            if self.troops[last_move.origin] == 1 {
                retain_different_origin(move_list, last_move.origin);
            }
        }
    }

    pub fn turn(&self) -> Turn {
        self.turn
    }

    pub fn troops(&self, territory: TerritoryId) -> u32 {
        self.troops[territory]
    }

    pub(crate) fn remove_troops(&mut self, territory: TerritoryId, count: u32) {
        assert!(count < self.troops[territory]);
        self.troops[territory] -= count;
    }

    pub fn occupier(&self, territory: TerritoryId) -> PlayerId {
        self.occupiers[territory]
    }

    pub fn territory_conquered(&self) -> bool {
        self.territory_conquered
    }

    pub fn players_eliminated(&self) -> u8 {
        self.players_eliminated
    }

    pub fn card_sets_redeemed(&self) -> u32 {
        self.card_sets_redeemed
    }

    pub fn troops_to_place(&self) -> u32 {
        self.troops_to_place
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Turn {
    Player,
    Chance(PlayerMove),
    PlaceTroops,
}

impl Turn {
    /// Returns `true` if the turn is [`Player`].
    ///
    /// [`Player`]: Turn::Player
    #[must_use]
    pub fn is_player(&self) -> bool {
        matches!(self, Self::Player)
    }

    /// Returns `true` if the turn is [`Chance`].
    ///
    /// [`Chance`]: Turn::Chance
    #[must_use]
    pub fn is_chance(&self) -> bool {
        matches!(self, Self::Chance(..))
    }

    /// Returns `true` if the turn is [`PlaceTroops`].
    ///
    /// [`PlaceTroops`]: Turn::PlaceTroops
    #[must_use]
    pub fn is_place_troops(&self) -> bool {
        matches!(self, Self::PlaceTroops)
    }
}

#[cfg(target_family = "wasm")]
mod wasm;

#[cfg(target_family = "wasm")]
pub(crate) use wasm::*;

#[cfg(not(target_family = "wasm"))]
pub(crate) fn retain_different_origin(moves: &mut Vec<PlayerMove>, origin: TerritoryId) {
    moves.retain(|x| x.origin != origin);
}

#[cfg(not(target_family = "wasm"))]
pub(crate) fn retain_different_dest(moves: &mut Vec<PlayerMove>, dest: TerritoryId) {
    moves.retain(|x| x.dest != dest);
}

#[cfg(not(target_family = "wasm"))]
pub(crate) fn retain_different_origin_dest(
    moves: &mut Vec<PlayerMove>,
    origin: TerritoryId,
    dest: TerritoryId,
) {
    moves.retain(|x| x.origin != origin && x.dest != dest);
}
