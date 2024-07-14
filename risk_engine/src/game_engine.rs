use enum_map::EnumMap;
use rand::prelude::SliceRandom;
use risk_shared::{
    player::{Player, PlayerBot, PlayerId},
    record::{Cause, Move, Record, StartGame, TerritoryConquered},
};

use crate::{
    player::PlayerConnection,
    state::{record, EngineState},
    MAX_GAME_RECORDING_SIZE, NUM_STARTING_TROOPS,
};

pub enum GameResult {
    Cancelled,
    Success(PlayerId),
}

pub struct GameEngine {
    players: EnumMap<PlayerId, PlayerConnection<Box<dyn PlayerBot>>>,
    state: EngineState,
}

impl GameEngine {
    pub fn new(players: EnumMap<PlayerId, PlayerConnection<Box<dyn PlayerBot>>>) -> Self {
        GameEngine {
            state: EngineState::new(),
            players,
        }
    }

    pub fn start(&mut self) -> GameResult {
        for player in self.players.values_mut() {
            player.reset();
        }

        self.run_game()
    }

    fn run_game(&mut self) -> GameResult {
        let turn_order = {
            let mut turn_order = PlayerId::ALL;
            turn_order.shuffle(&mut rand::thread_rng());
            turn_order
        };

        self.state.commit(Record::StartGame(Box::new(StartGame {
            turn_order,
            players: EnumMap::from_fn(|x| Player::new(x, NUM_STARTING_TROOPS)),
        })));

        self.state.commit(Record::ShuffledCards);

        self.start_claim_territories_phase();
        self.start_place_initial_troops_phase();

        let mut next_turn = 0;
        while self.state.players().values().filter(|x| x.alive).count() > 1 {
            if self.state.recording().len() >= MAX_GAME_RECORDING_SIZE {
                return GameResult::Cancelled;
            }

            let turn = loop {
                let turn = next_turn;
                next_turn = (next_turn + 1) % 5;

                let player_id = self.state.turn_order()[turn];
                if self.state.players()[player_id].alive {
                    break turn;
                }
            };

            self.troop_phase(turn);
            self.attack_phase(turn);

            if self.state.players().values().filter(|x| x.alive).count() > 1 {
                self.fortify_phase(turn);
            }
        }

        let winner = self.state.players().values().find(|x| x.alive).unwrap().id;
        self.state.commit(Record::Winner(winner));
        GameResult::Success(winner)
    }

    fn start_claim_territories_phase(&mut self) {
        let mut turn = 0;
        while self
            .state
            .territories()
            .values()
            .any(|x| x.occupier.is_none())
        {
            let player_id = self.state.turn_order()[turn];
            let connection = &mut self.players[player_id];
            turn = (turn + 1) % 5;

            let territory = connection.query_claim_territory(&self.state);
            self.state
                .commit(Record::Move(player_id, Move::ClaimTerritory(territory)));
        }
    }

    fn start_place_initial_troops_phase(&mut self) {
        let mut turn = 0;
        while self
            .state
            .players()
            .values()
            .any(|x| x.troops_remaining > 0)
        {
            let player_id = self.state.turn_order()[turn];
            let connection = &mut self.players[player_id];
            turn = (turn + 1) % 5;

            let player = &self.state.players()[player_id];
            if player.troops_remaining == 0 {
                continue;
            }

            let territory = connection.query_place_initial_troop(&self.state);
            self.state
                .commit(Record::Move(player_id, Move::PlaceInitialTroop(territory)));
        }
    }

    fn troop_phase(&mut self, turn: usize) {
        let player_id = self.state.turn_order()[turn];
        let connection = &mut self.players[player_id];

        self.state
            .commit(record::start_turn(&self.state, player_id));

        let response = connection.query_redeem_cards(&self.state, Cause::TurnStarted);
        self.state
            .commit(Record::Move(player_id, Move::RedeemCards(response)));

        let response = connection.query_distribute_troops(&self.state, Cause::TurnStarted);
        self.state
            .commit(Record::Move(player_id, Move::DistributeTroops(response)));
    }

    fn attack_phase(&mut self, turn: usize) {
        let mut conquered_territory = false;

        let player_id = self.state.turn_order()[turn];
        loop {
            let connection = &mut self.players[player_id];
            let attack = connection.query_attack(&self.state);
            self.state.commit(Record::move_attack(player_id, attack));
            let move_attack_id = self.state.recording().len() - 1;

            let Some(attack) = attack else {
                break;
            };

            let Some(defending_player) =
                self.state.territories()[attack.defending_territory].occupier
            else {
                panic!("Tried to attack unoccupied territory.");
            };

            let defend = self.players[defending_player].query_defend(&self.state, move_attack_id);
            self.state
                .commit(Record::Move(defending_player, Move::Defend(defend)));
            let move_defend_id = self.state.recording().len() - 1;

            let record_attack = record::attack(&self.state, move_attack_id, move_defend_id);
            self.state.commit(Record::Attack(record_attack));
            let record_attack_id = self.state.recording().len() - 1;

            if record_attack.territory_conquered {
                conquered_territory = true;
                let record = TerritoryConquered { record_attack_id };

                self.state.commit(Record::TerritoryConquered(record));
            }

            if record_attack.defender_eliminated {
                let record =
                    record::player_eliminated(&self.state, record_attack_id, defending_player);
                self.state.commit(Record::PlayerEliminated(record));

                if self.state.players().values().filter(|x| x.alive).count() == 1 {
                    return;
                }
            }

            let connection = &mut self.players[player_id];
            // Move troops after attack
            if record_attack.territory_conquered {
                let response = connection.query_troops_after_attack(&self.state, record_attack_id);
                self.state.commit(Record::Move(
                    player_id,
                    Move::MoveTroopsAfterAttack(response),
                ));
            }

            if record_attack.defender_eliminated && self.state.players()[player_id].cards.len() > 6
            {
                let response = connection.query_redeem_cards(&self.state, Cause::PlayerEliminated);
                self.state
                    .commit(Record::Move(player_id, Move::RedeemCards(response)));

                let response =
                    connection.query_distribute_troops(&self.state, Cause::PlayerEliminated);
                self.state
                    .commit(Record::Move(player_id, Move::DistributeTroops(response)));
            }
        }

        if conquered_territory {
            if self.state.deck().is_empty() {
                self.state.commit(Record::ShuffledCards);
            }

            let record = record::drew_card(&mut self.state, player_id);
            self.state.commit(Record::DrewCard(record));
        }
    }

    fn fortify_phase(&mut self, turn: usize) {
        let player_id = self.state.turn_order()[turn];
        let connection = &mut self.players[player_id];
        let response = connection.query_fortify(&self.state);
        self.state.commit(Record::move_fortify(player_id, response));
    }
}
