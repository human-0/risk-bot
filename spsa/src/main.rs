use attack_game::{
    evaluate::{self, Eval},
    strategy::puct,
};
use enum_map::EnumMap;
use rand::{RngCore, SeedableRng};
use risk_bots::strategy::{Params, PuctBot};
use risk_helper::ManagedPlayerBot;
use spsa::{
    eval_params, float_params,
    spsa::{HyperParams, Spsa},
    CreateFromParams,
};

fn main() {
    let hyper_params = HyperParams::new(10000, 4);

    // curr_value, min, max, c_end, r_end
    let float_params = float_params! {
        (0.0, 1.0, 0.05, 0.01) => {
            first_enemy_troop_reduction: 0.653307917196393,
            first_friendly_troop_reduction: 0.7108603406436419,
            puct_first_enemy_troop_reduction: 0.19112898155428668,
            puct_first_friendly_troop_reduction: 0.5879683842636655,
        }
        (0.0, 10.0, 0.05, 0.01) => {
            c_puct: 0.7390775750168702,
            c_puct_troops: 0.6886480396817488,
        }
        (0.0, 10.0, 0.01, 0.01) => {
            resolve_k: 0.14864234033422372,
        }
    };

    let eval_params = eval_params! {
        (0.0, 10.0, 0.005, 0.01) => {
            territory_occupied: Eval(0.003315954179343992, 0.01733528843180152),
        }
        (-10.0, 0.0, 0.005, 0.01) => {
            weak_territory: Eval(-0.0714311161391381, -0.0037244297863779164),
            isolated_territory: Eval(-0.00040022064362791996, -0.06569914467196432),
        }
        (0.0, 10.0, 0.05, 0.01) => {
            player_eliminated: Eval(0.9919550758267683, 1.5722449290385963),
            territory_conquered: Eval(0.4724614848987875, 0.35789063202899885),
            troop_count: Eval(0.926231230440439, 1.338092013414867),
            cont_0_p0: Eval(0.9558890103221532, 0.15098427931299677),
            cont_1_p0: Eval(0.7277694569181395, 0.0666996757872502),
            cont_2_p0: Eval(0.9326679208533363, 0.33625543658410884),
            cont_3_p0: Eval(0.6307224614377632, 0.6701743236482217),
            cont_4_p0: Eval(0.32523546520730007, 0.09992925264161054),
            cont_5_p0: Eval(0.8203591444204165, 0.08574972678381543),
        }
        (-10.0, 0.0, 0.05, 0.01) => {
            cont_0_p1: Eval(-1.6776350202078967, -0.15164548820984267),
            cont_0_p2: Eval(-1.3765896881734276, -0.232673138969994),
            cont_0_p3: Eval(-0.3384658254959926, -0.14901338140369416),
            cont_0_p4: Eval(-0.3717108258748576, -0.061237332536988306),
            cont_1_p1: Eval(-2.025908953786366, -0.3872655343701831),
            cont_1_p2: Eval(-1.499885023005234, -0.0005524694821560541),
            cont_1_p3: Eval(-0.38553733186005634, -0.49667975093608724),
            cont_1_p4: Eval(-0.17645178084387356, -0.32060523252239903),
            cont_2_p1: Eval(-1.9888719820782672, -0.6387179048622024),
            cont_2_p2: Eval(-1.1231484829501457, -0.6520227847420222),
            cont_2_p3: Eval(-0.10940900128741532, -0.15745123622142884),
            cont_2_p4: Eval(-0.0369167933802483, -0.3589365843367668),
            cont_3_p1: Eval(-1.3983444053811835, -0.6430636781471771),
            cont_3_p2: Eval(-0.8138698577841516, -0.598918910129025),
            cont_3_p3: Eval(-0.5431220819920496, -0.14213397636367478),
            cont_3_p4: Eval(-0.1189081449764817, -0.13227513709024483),
            cont_4_p1: Eval(-0.6910158095290883, -0.1398995847272377),
            cont_4_p2: Eval(-0.2301028220990661, -0.12131142983732943),
            cont_4_p3: Eval(-0.10723667309111082, -0.2880672439340498),
            cont_4_p4: Eval(-0.016275346914563716, -0.07026083121554327),
            cont_5_p1: Eval(-1.1552784134901546, -0.1598721191956807),
            cont_5_p2: Eval(-0.3385357443536904, -0.18604620180146417),
            cont_5_p3: Eval(-0.10472990623261633, -0.12217723691458308),
            cont_5_p4: Eval(-0.17740899398813584, -0.17179401833242883),
        }
        (-10.0, 10.0, 0.05, 0.01) => {
            bias: Eval(-0.5433299644421017, -0.2201531424695424),
        }
    };

    let params = float_params
        .into_iter()
        .chain(eval_params)
        .map(|(key, value)| {
            (
                key.to_owned(),
                hyper_params.make_params(value.0, value.1, value.2, value.3, value.4),
            )
        })
        .collect();

    println!("Params: {params:#?}");

    let mut spsa = Spsa::new(params, hyper_params);
    let result = spsa.tune(&SpsaPuct, rand::thread_rng(), "tune.spsa");

    let mut values = result.into_iter().collect::<Vec<_>>();
    values.sort_by(|x, y| x.0.cmp(&y.0));
    for (key, value) in values {
        println!("{key}: {value}");
    }
}

struct SpsaPuct;

impl CreateFromParams for SpsaPuct {
    type Bot = ManagedPlayerBot<PuctBot<'static, rand_xoshiro::Xoshiro256StarStar>>;
    fn create_from_params(&self, params: &std::collections::HashMap<String, f64>) -> Self::Bot {
        let eval_params = evaluate::Params {
            territory_occupied: Eval(
                params["territory_occupied_0"],
                params["territory_occupied_1"],
            ),
            weak_territory: Eval(params["weak_territory_0"], params["weak_territory_1"]),
            isolated_territory: Eval(
                params["isolated_territory_0"],
                params["isolated_territory_1"],
            ),
            troop_count: Eval(params["troop_count_0"], params["troop_count_1"]),
            player_eliminated: Eval(params["player_eliminated_0"], params["player_eliminated_1"]),
            continent_by_player: EnumMap::from_fn(|c| {
                EnumMap::from_fn(|p| {
                    Eval(
                        params[&format!("cont_{}_p{}_0", c as u8, p as u8)],
                        params[&format!("cont_{}_p{}_1", c as u8, p as u8)],
                    )
                })
            }),
            territory_conquered: Eval(
                params["territory_conquered_0"],
                params["territory_conquered_1"],
            ),
            bias: Eval(params["bias_0"], params["bias_1"]),
            resolve_k: params["resolve_k"],
        };

        let puct_params = puct::Params {
            c_puct: params["c_puct"],
            c_puct_troops: params["c_puct_troops"],
            first_enemy_troop_reduction: params["puct_first_enemy_troop_reduction"],
            first_friendly_troop_reduction: params["puct_first_friendly_troop_reduction"],
            eval: eval_params,
        };

        let params = Params {
            first_enemy_troop_reduction: params["first_enemy_troop_reduction"],
            first_friendly_troop_reduction: params["first_friendly_troop_reduction"],
            strategy_params: puct_params,
        };

        let bot = PuctBot::with_params(
            params,
            rand_xoshiro::Xoshiro256StarStar::seed_from_u64(rand::thread_rng().next_u64()),
        );

        ManagedPlayerBot::new(bot)
    }
}
