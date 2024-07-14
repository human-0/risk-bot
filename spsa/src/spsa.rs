use std::collections::HashMap;

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

use crate::CreateFromParams;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SpsaParam {
    pub curr_value: f64,
    pub min: f64,
    pub max: f64,
    pub c: f64,
    pub a: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HyperParams {
    alpha: f64,
    gamma: f64,
    a_ratio: f64,
    num_iterations: u64,
    games_per: u64,
}

impl HyperParams {
    pub fn new(num_iterations: u64, games_per: u64) -> Self {
        Self {
            alpha: 0.602,
            gamma: 0.101,
            a_ratio: 0.1,
            num_iterations,
            games_per,
        }
    }

    pub fn make_params(
        &self,
        curr_value: f64,
        min: f64,
        max: f64,
        c_end: f64,
        r_end: f64,
    ) -> SpsaParam {
        let c = c_end * (self.num_iterations as f64).powf(self.gamma);
        let a_end = r_end * c_end.powi(2);
        let a = a_end * (self.num_iterations as f64 + self.a_ratio).powf(self.alpha);
        SpsaParam {
            curr_value,
            min,
            max,
            c,
            a,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Spsa {
    params: HashMap<String, SpsaParam>,
    hyper_params: HyperParams,
    curr_iteration: u64,
}

impl Spsa {
    pub fn new(params: HashMap<String, SpsaParam>, hyper_params: HyperParams) -> Self {
        Self {
            params,
            hyper_params,
            curr_iteration: 0,
        }
    }

    pub fn tune<T, R>(&mut self, create: &T, mut rng: R, save_file: &str) -> HashMap<String, f64>
    where
        T: CreateFromParams + Sync + 'static,
        R: rand::Rng,
    {
        while self.curr_iteration < self.hyper_params.num_iterations {
            let k = self.curr_iteration as f64;
            let delta = self
                .params
                .keys()
                .map(|x| (x.clone(), gen_delta(&mut rng)))
                .collect::<HashMap<_, _>>();

            let theta_plus = self
                .params
                .iter()
                .map(|(key, param)| {
                    let c_k = param.c / (k + 1.0).powf(self.hyper_params.gamma);
                    (
                        key.clone(),
                        (param.curr_value + c_k * delta[key]).clamp(param.min, param.max),
                    )
                })
                .collect::<HashMap<_, _>>();

            let theta_minus = self
                .params
                .iter()
                .map(|(key, param)| {
                    let c_k = param.c / (k + 1.0).powf(self.hyper_params.gamma);
                    (
                        key.clone(),
                        (param.curr_value - c_k * delta[key]).clamp(param.min, param.max),
                    )
                })
                .collect::<HashMap<_, _>>();

            let match_ = (0..self.hyper_params.games_per)
                .into_par_iter()
                .map(|_| play_game::<T>(create, &theta_plus, &theta_minus))
                .sum::<f64>();

            for (key, param) in self.params.iter_mut() {
                let a_k =
                    param.a / (k + 1.0 + self.hyper_params.a_ratio).powf(self.hyper_params.alpha);
                let c_k = param.c / (k + 1.0).powf(self.hyper_params.gamma);
                param.curr_value = (param.curr_value + a_k * match_ / (c_k * delta[key]))
                    .clamp(param.min, param.max);
            }

            self.curr_iteration += self.hyper_params.games_per;

            if self.curr_iteration % (5 * self.hyper_params.games_per) == 0 {
                println!("Iteration: {}", self.curr_iteration);
                let mut values = self.params.iter().collect::<Vec<_>>();
                values.sort_by(|x, y| x.0.cmp(y.0));
                for (key, param) in values {
                    println!("{key}: {}", param.curr_value);
                }

                std::fs::write(save_file, serde_json::to_string(self).unwrap()).unwrap();
            }
        }

        self.params
            .iter()
            .map(|(key, param)| (key.clone(), param.curr_value))
            .collect()
    }
}

fn play_game<T>(create: &T, params_a: &HashMap<String, f64>, params_b: &HashMap<String, f64>) -> f64
where
    T: CreateFromParams + 'static,
{
    let mut game = GameEngine::new(EnumMap::from_fn(|player| {
        let bot = match player {
            PlayerId::P0 => Box::new(create.create_from_params(params_a)) as Box<dyn PlayerBot>,
            PlayerId::P1 => Box::new(create.create_from_params(params_b)) as Box<dyn PlayerBot>,
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
    }));

    match game.start() {
        GameResult::Success(PlayerId::P0) => 1.0,
        GameResult::Success(PlayerId::P1) => -1.0,
        _ => 0.0,
    }
}

fn gen_delta(rng: &mut impl rand::Rng) -> f64 {
    if rng.gen_bool(0.5) {
        1.0
    } else {
        -1.0
    }
}
