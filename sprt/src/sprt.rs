use enum_map::EnumMap;
use rand::{RngCore, SeedableRng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use risk_bots::{very_bad::VeryBad, very_bad13::VeryBad13};
use risk_engine::{
    game_engine::{GameEngine, GameResult},
    player::PlayerConnection,
};
use risk_helper::ManagedPlayerBot;
use risk_shared::player::{PlayerBot, PlayerId};

use crate::CreatePlayerBot;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SprtParams {
    pub h0_elo: f64,
    pub h1_elo: f64,
    pub alpha: f64,
    pub beta: f64,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Sprt {
    h0_elo: f64,
    h1_elo: f64,
    a: f64,
    b: f64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub struct SprtResult {
    pub results: [u64; 3],
}

impl SprtResult {
    pub fn num_games(self) -> u64 {
        self.results.iter().sum::<u64>()
    }

    pub fn score(self) -> f64 {
        (self.results[0] as f64 + 0.5 * self.results[1] as f64) / self.num_games() as f64
    }

    pub fn elo_diff(self) -> f64 {
        let score = self.score();
        -400.0 * ((1.0 - score) / score).log10()
    }

    pub fn llr(self, h0_elo: f64, h1_elo: f64) -> f64 {
        if self.results[0] == 0 || self.results[2] == 0 {
            return 0.0;
        }

        let num_games = self.num_games();
        let probs = self.results.map(|x| x as f64 / num_games as f64);

        let draw_elo =
            200.0 * (((1.0 - probs[0]) / probs[0]) * ((1.0 - probs[2]) / probs[2])).log10();

        let prob_h0 = elo_probs(h0_elo, draw_elo);
        let prob_h1 = elo_probs(h1_elo, draw_elo);

        (0..3)
            .map(|x| {
                // Avoid nans
                if self.results[x] == 0 {
                    0.0
                } else {
                    self.results[x] as f64 * (prob_h1[x] / prob_h0[x]).ln()
                }
            })
            .sum::<f64>()
    }
}

impl Sprt {
    pub fn new(params: SprtParams) -> Self {
        Self {
            h0_elo: params.h0_elo,
            h1_elo: params.h1_elo,
            a: (params.beta / (1.0 - params.alpha)).ln(),
            b: ((1.0 - params.beta) / params.alpha).ln(),
        }
    }

    pub fn sprt<P1, P2>(&self, p1: &P1, p2: &P2, batch_size: u64, write_file: &str) -> SprtResult
    where
        P1: CreatePlayerBot + Sync + 'static,
        P2: CreatePlayerBot + Sync + 'static,
    {
        let mut results = SprtResult { results: [0; 3] };

        loop {
            let match_ = (0..batch_size)
                .into_par_iter()
                .map(|_| play_game(p1, p2))
                .reduce(
                    || [0; 3],
                    |[x1, y1, z1], [x2, y2, z2]| [x1 + x2, y1 + y2, z1 + z2],
                );

            results.results[0] += match_[0];
            results.results[1] += match_[1];
            results.results[2] += match_[2];

            if results.num_games() % (5 * batch_size) == 0 {
                println!(
                    "{} Games: {:?} Score: {:.2}% Elo: {} LLR: {}",
                    results.num_games(),
                    results.results,
                    results.score() * 100.0,
                    results.elo_diff(),
                    results.llr(self.h0_elo, self.h1_elo)
                );

                std::fs::write(write_file, serde_json::to_string(&results).unwrap()).unwrap()
            }

            if !(self.a..=self.b).contains(&results.llr(self.h0_elo, self.h1_elo)) {
                break;
            }
        }

        results
    }
}

fn play_game<P1, P2>(p1: &P1, p2: &P2) -> [u64; 3]
where
    P1: CreatePlayerBot + 'static,
    P2: CreatePlayerBot + 'static,
{
    let players = EnumMap::from_fn(|player| {
        let bot = match player {
            PlayerId::P0 => Box::new(p1.create()) as Box<dyn PlayerBot>,
            PlayerId::P1 => Box::new(p2.create()) as Box<dyn PlayerBot>,
            PlayerId::P2 => Box::new(ManagedPlayerBot::new(VeryBad::new())),
            PlayerId::P3 => Box::new(ManagedPlayerBot::new(VeryBad13::new())),
            PlayerId::P4 => {
                let rng =
                    rand_xoshiro::Xoshiro256StarStar::seed_from_u64(rand::thread_rng().next_u64());

                let complex = risk_bots::complex::ComplexExample::new(rng);
                Box::new(ManagedPlayerBot::new(complex)) as Box<dyn PlayerBot>
            }
        };
        PlayerConnection::new(bot, player)
    });

    let mut game = GameEngine::new(players);

    match game.start() {
        GameResult::Success(PlayerId::P0) => [1, 0, 0],
        GameResult::Success(PlayerId::P1) => [0, 0, 1],
        _ => [0, 1, 0],
    }
}

fn elo_probs(elo: f64, draw_elo: f64) -> [f64; 3] {
    let win = 1.0 / (1.0 + 10.0_f64.powf((-elo + draw_elo) / 400.0));
    let loss = 1.0 / (1.0 + 10.0_f64.powf((elo + draw_elo) / 400.0));

    [win, 1.0 - win - loss, loss]
}
