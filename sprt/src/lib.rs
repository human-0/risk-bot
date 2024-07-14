pub mod sprt;

use risk_shared::player::PlayerBot;

pub trait CreatePlayerBot {
    type Bot: PlayerBot;

    fn create(&self) -> Self::Bot;
}
