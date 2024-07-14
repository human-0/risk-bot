use connection::Connection;
use risk_shared::{
    player::{PlayerBot, PlayerId},
    record::PublicRecord,
};

pub mod connection;

pub struct JsonGame<P: PlayerBot> {
    player: P,
    player_id: PlayerId,
    connection: Connection,
}

impl<P: PlayerBot> JsonGame<P> {
    pub fn new(player: P) -> Self {
        Self {
            player,
            player_id: PlayerId::P0,
            connection: Connection::new().unwrap(),
        }
    }

    pub fn run(mut self) {
        let query = self.connection.get_next_query();
        let Some((0, PublicRecord::PublicStartGame(start))) = query.update.enumerate_items().next()
        else {
            unreachable!();
        };

        self.player_id = start.you.id;
        let mov = self.player.query(query);
        self.connection.send_move(self.player_id, mov);

        loop {
            let query = self.connection.get_next_query();
            let mov = self.player.query(query);
            self.connection.send_move(self.player_id, mov);
        }
    }
}
