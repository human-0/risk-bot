use rand::Rng;
use risk_shared::{
    map::Continent,
    player::PlayerId,
    record::{Attack, DrewCard, Move, PlayerEliminated, Record, StartTurn},
};

use super::EngineState;

pub fn attack(state: &EngineState, move_attack_id: usize, move_defend_id: usize) -> Attack {
    let Record::Move(_, Move::Attack(move_attack)) = &state.recording[move_attack_id] else {
        unreachable!()
    };

    let attacking_troops = move_attack.attacking_troops;

    let Record::Move(defending_player, Move::Defend(move_defend)) =
        &state.recording[move_defend_id]
    else {
        unreachable!()
    };

    let defending_troops = move_defend.defending_troops;

    let attacking_rolls = roll_sorted(attacking_troops);
    let defending_rolls = roll_sorted(defending_troops);

    let attacking_lost = attacking_rolls
        .iter()
        .zip(&defending_rolls)
        .filter(|(x, y)| x <= y)
        .count() as u32;
    let defending_lost = attacking_rolls
        .iter()
        .zip(&defending_rolls)
        .filter(|(x, y)| x > y)
        .count() as u32;

    let defending_territory = &state.territories[move_attack.defending_territory];
    let territory_conquered = defending_lost == defending_territory.troops;
    let defender_eliminated = territory_conquered
        && !state.territories.iter().any(|(id, t)| {
            t.occupier == Some(*defending_player) && id != move_attack.defending_territory
        });

    Attack {
        move_attack_id,
        move_defend_id,
        attacking_lost,
        defending_lost,
        territory_conquered,
        defender_eliminated,
    }
}

pub fn player_eliminated(
    state: &EngineState,
    record_attack_id: usize,
    player: PlayerId,
) -> PlayerEliminated {
    let cards_surrendered = state.players[player].cards.clone();
    PlayerEliminated {
        player,
        record_attack_id,
        cards_surrendered,
    }
}

pub fn start_turn(state: &EngineState, player: PlayerId) -> Record {
    let player_territories = state
        .territories
        .iter()
        .filter(|(_, territory)| territory.occupier == Some(player))
        .map(|(id, _)| id)
        .collect::<Vec<_>>();

    let territory_bonus = std::cmp::max(3, player_territories.len() as u32 / 3);
    let continents_held = Continent::ALL
        .into_iter()
        .filter(|c| {
            c.iter_territories()
                .all(|t| state.territories[t].occupier == Some(player))
        })
        .collect::<Vec<_>>();

    let continent_bonus = continents_held.iter().copied().map(Continent::bonus).sum();

    Record::StartTurn(StartTurn {
        player,
        continents_held,
        territories_held: player_territories.len() as u32,
        continent_bonus,
        territory_bonus,
    })
}

pub fn drew_card(state: &mut EngineState, player: PlayerId) -> DrewCard {
    DrewCard {
        player,
        card: state.draw_card(),
    }
}

fn roll_sorted(count: u32) -> Vec<u32> {
    let mut attacking_roles = std::iter::repeat_with(|| rand::thread_rng().gen_range(1..=6))
        .take(count as usize)
        .collect::<Vec<_>>();

    attacking_roles.sort_unstable_by_key(|&x| std::cmp::Reverse(x));
    attacking_roles
}
