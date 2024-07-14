use risk_shared::{
    player::PlayerId,
    record::{Move, PublicPlayerEliminated, PublicRecord, Record},
};

use crate::state::EngineState;

pub fn censor(state: &EngineState, record: Record, player: PlayerId) -> PublicRecord {
    match record {
        Record::Attack(r) => PublicRecord::Attack(r),
        Record::DrewCard(r) => {
            if r.player == player {
                PublicRecord::DrewCard(r)
            } else {
                PublicRecord::PublicDrewCard(r.player)
            }
        }
        Record::PlayerEliminated(r) => {
            let Record::Attack(record) = state.recording()[r.record_attack_id] else {
                unreachable!();
            };

            let Record::Move(attacking_player, Move::Attack(_)) =
                state.recording()[record.move_attack_id]
            else {
                unreachable!();
            };

            if attacking_player == player {
                PublicRecord::PlayerEliminated(r)
            } else {
                PublicRecord::PublicPlayerEliminated(PublicPlayerEliminated {
                    player: r.player,
                    record_attack_id: r.record_attack_id,
                    cards_surrendered_count: r.cards_surrendered.len(),
                })
            }
        }
        Record::RedeemedCards(r) => PublicRecord::RedeemedCards(r),
        Record::ShuffledCards => PublicRecord::ShuffledCards,
        Record::StartGame(r) => PublicRecord::PublicStartGame(Box::new(r.censor(player))),
        Record::StartTurn(r) => PublicRecord::StartTurn(r),
        Record::TerritoryConquered(r) => PublicRecord::TerritoryConquered(r),
        Record::Winner(r) => PublicRecord::Winner(r),
        Record::Move(p, r) => PublicRecord::Move(p, r),
    }
}
