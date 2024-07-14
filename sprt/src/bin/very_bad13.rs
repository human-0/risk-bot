use risk_bots::{very_bad::VeryBad, very_bad13::VeryBad13};
use risk_helper::ManagedPlayerBot;
use sprt::{
    sprt::{Sprt, SprtParams},
    CreatePlayerBot,
};

fn main() {
    let params = SprtParams {
        h0_elo: 0.0,
        h1_elo: 5.0,
        alpha: 0.05,
        beta: 0.05,
    };

    let sprt = Sprt::new(params);
    let results = sprt.sprt(&Dev, &Base, 4, "complex.sprt");

    println!(
        "{} Games: {:?} Score: {:.2}% Elo: {} LLR: {}",
        results.num_games(),
        results.results,
        results.score() * 100.0,
        results.elo_diff(),
        results.llr(params.h0_elo, params.h1_elo),
    );
}

struct Base;

impl CreatePlayerBot for Base {
    type Bot = ManagedPlayerBot<VeryBad>;

    fn create(&self) -> Self::Bot {
        ManagedPlayerBot::new(VeryBad::new())
    }
}

struct Dev;

impl CreatePlayerBot for Dev {
    type Bot = ManagedPlayerBot<VeryBad13>;

    fn create(&self) -> Self::Bot {
        ManagedPlayerBot::new(VeryBad13::new())
    }
}
