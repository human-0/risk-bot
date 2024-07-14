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
            territory_occupied: Eval(0.003315954179343992, 0.01733528843180152),
            weak_territory: Eval(-0.0714311161391381, -0.0037244297863779164),
            isolated_territory: Eval(-0.00040022064362791996, -0.06569914467196432),
            player_eliminated: Eval(0.9919550758267683, 1.5722449290385963),
            territory_conquered: Eval(0.4724614848987875, 0.35789063202899885),
            troop_count: Eval(0.926231230440439, 1.338092013414867),
            bias: Eval(-0.5433299644421017, -0.2201531424695424),
            resolve_k: 0.14864234033422372,
            continent_by_player: EnumMap::from_array([
                EnumMap::from_array([
                    Eval(0.9558890103221532, 0.15098427931299677),
                    Eval(-1.6776350202078967, -0.15164548820984267),
                    Eval(-1.3765896881734276, -0.232673138969994),
                    Eval(-0.3384658254959926, -0.14901338140369416),
                    Eval(-0.3717108258748576, -0.061237332536988306),
                ]),
                EnumMap::from_array([
                    Eval(0.7277694569181395, 0.0666996757872502),
                    Eval(-2.025908953786366, -0.3872655343701831),
                    Eval(-1.499885023005234, -0.0005524694821560541),
                    Eval(-0.38553733186005634, -0.49667975093608724),
                    Eval(-0.17645178084387356, -0.32060523252239903),
                ]),
                EnumMap::from_array([
                    Eval(0.9326679208533363, 0.33625543658410884),
                    Eval(-1.9888719820782672, -0.6387179048622024),
                    Eval(-1.1231484829501457, -0.6520227847420222),
                    Eval(-0.10940900128741532, -0.15745123622142884),
                    Eval(-0.0369167933802483, -0.3589365843367668),
                ]),
                EnumMap::from_array([
                    Eval(0.6307224614377632, 0.6701743236482217),
                    Eval(-1.3983444053811835, -0.6430636781471771),
                    Eval(-0.8138698577841516, -0.598918910129025),
                    Eval(-0.5431220819920496, -0.14213397636367478),
                    Eval(-0.1189081449764817, -0.13227513709024483),
                ]),
                EnumMap::from_array([
                    Eval(0.32523546520730007, 0.09992925264161054),
                    Eval(-0.6910158095290883, -0.1398995847272377),
                    Eval(-0.2301028220990661, -0.12131142983732943),
                    Eval(-0.10723667309111082, -0.2880672439340498),
                    Eval(-0.016275346914563716, -0.07026083121554327),
                ]),
                EnumMap::from_array([
                    Eval(0.8203591444204165, 0.08574972678381543),
                    Eval(-1.1552784134901546, -0.1598721191956807),
                    Eval(-0.3385357443536904, -0.18604620180146417),
                    Eval(-0.10472990623261633, -0.12217723691458308),
                    Eval(-0.17740899398813584, -0.17179401833242883),
                ]),
            ]),
        };

        let puct_params = puct::Params {
            c_puct: 0.7390775750168702,
            c_puct_troops: 0.6886480396817488,
            first_enemy_troop_reduction: 0.19112898155428668,
            first_friendly_troop_reduction: 0.5879683842636655,
            eval: eval_params,
        };

        let params = Params {
            strategy_params: puct_params,
            first_enemy_troop_reduction: 0.653307917196393,
            first_friendly_troop_reduction: 0.7108603406436419,
        };

        let rng = rand_xoshiro::Xoshiro256StarStar::seed_from_u64(rand::thread_rng().next_u64());
        ManagedPlayerBot::new(PuctBot::with_params(params, rng))
    }
}
