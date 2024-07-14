use attack_game::{
    evaluate::{self, Eval},
    strategy::puct,
};
use enum_map::EnumMap;
use rand::{RngCore, SeedableRng};
use risk_bots::strategy::{Params, PuctBot};
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
    let results = sprt.sprt(&Dev, &Base, 4, "test.sprt");

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
    type Bot = ManagedPlayerBot<PuctBot<'static, rand_xoshiro::Xoshiro256StarStar>>;

    fn create(&self) -> Self::Bot {
        let rng = rand_xoshiro::Xoshiro256StarStar::seed_from_u64(rand::thread_rng().next_u64());
        ManagedPlayerBot::new(PuctBot::new(rng))
    }
}

struct Dev;

impl CreatePlayerBot for Dev {
    type Bot = ManagedPlayerBot<PuctBot<'static, rand_xoshiro::Xoshiro256StarStar>>;

    fn create(&self) -> Self::Bot {
        let eval_params = evaluate::Params {
            territory_occupied: Eval(0.004602183368215947, 0.013104483261206624),
            weak_territory: Eval(-0.06986899701902574, -0.006039317807500187),
            isolated_territory: Eval(-0.0023445806366162577, -0.08354679434136691),
            player_eliminated: Eval(1.2571525464830933, 1.8348492592151964),
            territory_conquered: Eval(0.251873746041146, 0.33770224917234126),
            troop_count: Eval(0.9379646647806544, 1.267830540861828),
            resolve_k: 0.14662364924017113,
            bias: Eval(-0.6038499097696227, -0.36524733244238594),
            continent_by_player: EnumMap::from_array([
                EnumMap::from_array([
                    Eval(1.0694851061151829, 0.06441334684074566),
                    Eval(-1.5597025025673659, -0.15745570400215952),
                    Eval(-1.6228947105965463, -0.11907015330387281),
                    Eval(-0.33870410392482064, -0.1364071630400263),
                    Eval(-0.5356322184801511, -0.10317302189366849),
                ]),
                EnumMap::from_array([
                    Eval(0.5491656406773419, 0.37041034537987116),
                    Eval(-2.1910144018747886, -0.368342577692111),
                    Eval(-1.4249260276555964, -0.005275070987203928),
                    Eval(-0.18566724389644645, -0.6451241430352684),
                    Eval(-0.08033828887263685, -0.42379664960382146),
                ]),
                EnumMap::from_array([
                    Eval(0.8047103607875169, 0.18521893217969115),
                    Eval(-1.9735927542878395, -0.6161537374725065),
                    Eval(-1.1728623532918598, -0.4978172218341361),
                    Eval(-0.053214523047240135, -0.15567833749883137),
                    Eval(-0.09133174485266425, -0.3255621448024393),
                ]),
                EnumMap::from_array([
                    Eval(0.5046775660789545, 0.505476133893031),
                    Eval(-1.4440393129525158, -0.48812430672181506),
                    Eval(-0.8993329617218632, -0.5935821029043132),
                    Eval(-0.5023216419717192, -0.28579123517483485),
                    Eval(-0.31250103320655964, -0.06674038325837807),
                ]),
                EnumMap::from_array([
                    Eval(0.4735398110186245, 0.24255722776766536),
                    Eval(-0.909810061036103, -0.10508908393099618),
                    Eval(-0.3640056552462979, -0.20906185192844956),
                    Eval(-0.4112714749276394, -0.15229075557518612),
                    Eval(-0.0700342694809049, -0.2047943403306252),
                ]),
                EnumMap::from_array([
                    Eval(0.7927570999552106, 0.01368859451191441),
                    Eval(-1.1850267624497925, -0.017359392018905993),
                    Eval(-0.26344912884447114, -0.26016772288891943),
                    Eval(-0.2153862156931699, -0.11649649242733816),
                    Eval(-0.32683830722755913, -0.06966700121303006),
                ]),
            ]),
        };

        let puct_params = puct::Params {
            c_puct: 0.6028558951476736,
            c_puct_troops: 0.702745173702057,
            first_enemy_troop_reduction: 0.13864357808607813,
            first_friendly_troop_reduction: 0.5888177056325355,
            eval: eval_params,
        };

        let params = Params {
            strategy_params: puct_params,
            first_enemy_troop_reduction: 0.47578774202200713,
            first_friendly_troop_reduction: 0.9711985622851357,
        };

        let rng = rand_xoshiro::Xoshiro256StarStar::seed_from_u64(rand::thread_rng().next_u64());
        ManagedPlayerBot::new(PuctBot::with_params(params, rng))
    }
}
